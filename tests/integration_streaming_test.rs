// Remove unused import
use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

mod common;

use common::{
    collect_stream_messages, create_concurrent_streaming_clients, init_test_logging,
    validate_rfc3339_timestamp, StreamPerformanceMonitor, StreamingClient, TestServer,
};

#[tokio::test]
async fn test_streaming_basic_functionality() {
    init_test_logging();

    // Start test server
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Create streaming client
    let mut client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    // Start time stream
    let mut stream = client
        .start_time_stream()
        .await
        .expect("Failed to start time stream");

    // Collect first few messages
    let messages = collect_stream_messages(&mut stream, 3, Duration::from_secs(2))
        .await
        .expect("Failed to collect stream messages");

    assert_eq!(messages.len(), 3);

    // Validate all timestamps are valid RFC3339
    for message in &messages {
        assert!(validate_rfc3339_timestamp(&message.timestamp));
        assert!(!message.timestamp.is_empty());
    }

    // All timestamps should be different (streaming at 1-second intervals)
    assert_ne!(messages[0].timestamp, messages[1].timestamp);
    assert_ne!(messages[1].timestamp, messages[2].timestamp);
}

#[tokio::test]
async fn test_streaming_client_disconnection() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let mut client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    let mut stream = client
        .start_time_stream()
        .await
        .expect("Failed to start time stream");

    // Get first message to ensure stream starts
    let first_message = timeout(Duration::from_secs(2), stream.next()).await;
    assert!(first_message.is_ok());
    assert!(first_message.unwrap().is_some());

    // Simulate client disconnection by dropping the stream
    drop(stream);
    drop(client);

    // Give time for server-side cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Server should continue running and accept new connections
    let mut new_client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create new streaming client after disconnection");

    let mut new_stream = new_client
        .start_time_stream()
        .await
        .expect("Failed to start new stream after disconnection");

    let new_message = timeout(Duration::from_secs(2), new_stream.next()).await;
    assert!(new_message.is_ok());
    assert!(new_message.unwrap().is_some());
}

#[tokio::test]
async fn test_streaming_multiple_concurrent_clients() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Create multiple concurrent streaming clients
    let mut clients = create_concurrent_streaming_clients(server.grpc_address(), 3)
        .await
        .expect("Failed to create concurrent clients");

    // Start streams for all clients
    let mut streams = Vec::new();
    for client in &mut clients {
        let stream = client
            .start_time_stream()
            .await
            .expect("Failed to start stream");
        streams.push(stream);
    }

    // Collect messages from all streams concurrently
    let mut handles = Vec::new();
    for (i, mut stream) in streams.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            let messages = collect_stream_messages(&mut stream, 2, Duration::from_secs(3)).await;
            (i, messages)
        });
        handles.push(handle);
    }

    // Wait for all streams to produce messages
    let mut results = Vec::new();
    for handle in handles {
        let (client_id, messages_result) = handle.await.expect("Task panicked");
        let messages = messages_result
            .unwrap_or_else(|e| panic!("Client {} failed to get messages: {}", client_id, e));
        results.push((client_id, messages));
    }

    // Verify all clients received messages
    assert_eq!(results.len(), 3);

    for (client_id, messages) in results {
        assert_eq!(
            messages.len(),
            2,
            "Client {} should have received 2 messages",
            client_id
        );

        // All timestamps should be valid
        for message in &messages {
            assert!(validate_rfc3339_timestamp(&message.timestamp));
        }

        // Messages from same client should be different
        assert_ne!(messages[0].timestamp, messages[1].timestamp);
    }
}

#[tokio::test]
async fn test_streaming_server_restart_recovery() {
    init_test_logging();

    // This test simulates what happens when a client tries to reconnect after server issues
    {
        let server = TestServer::start()
            .await
            .expect("Failed to start test server");
        let addr = server.grpc_address();

        // Create client and get one message
        let mut client = StreamingClient::connect(addr)
            .await
            .expect("Failed to create streaming client");

        let mut stream = client
            .start_time_stream()
            .await
            .expect("Failed to start time stream");

        let first_message = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(first_message.is_ok());

        drop(stream);
        drop(client);

        // Let server shut down (by dropping it)
        drop(server);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Start new server on same address (simulating restart)
    let new_server = TestServer::start()
        .await
        .expect("Failed to start new test server");

    // Client should be able to reconnect to new server
    let mut new_client = StreamingClient::from_test_server(&new_server)
        .await
        .expect("Failed to reconnect to new server");

    let mut new_stream = new_client
        .start_time_stream()
        .await
        .expect("Failed to start stream on new server");

    let message = timeout(Duration::from_secs(2), new_stream.next()).await;
    assert!(message.is_ok());
    assert!(message.unwrap().is_some());
}

#[tokio::test]
async fn test_streaming_performance_sustained_connection() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let mut client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    let mut stream = client
        .start_time_stream()
        .await
        .expect("Failed to start time stream");

    let mut monitor = StreamPerformanceMonitor::new();

    // Collect messages for a sustained period (10 messages = ~10 seconds)
    let message_count = 10;
    for i in 0..message_count {
        let message_result = timeout(Duration::from_secs(3), stream.next()).await;

        match message_result {
            Ok(Some(Ok(message))) => {
                monitor.record_message();

                // Validate message format
                assert!(validate_rfc3339_timestamp(&message.timestamp));

                println!(
                    "Message {}: {} (drift: {:.1}ms avg)",
                    i + 1,
                    message.timestamp,
                    monitor.average_timing_drift_ms()
                );
            }
            Ok(Some(Err(e))) => {
                panic!("Stream error on message {}: {}", i, e);
            }
            Ok(None) => {
                panic!("Stream ended unexpectedly after {} messages", i);
            }
            Err(_) => {
                panic!("Timeout waiting for message {} after 3 seconds", i);
            }
        }
    }

    // Verify performance characteristics
    assert_eq!(monitor.total_messages(), message_count);

    let messages_per_second = monitor.messages_per_second();
    println!("Performance: {:.2} messages/second", messages_per_second);

    // Should be close to 1 message per second (allowing for some variation)
    assert!(
        (0.8..=1.2).contains(&messages_per_second),
        "Expected ~1 msg/sec, got {:.2}",
        messages_per_second
    );

    // Timing drift should be reasonable (less than 100ms average)
    let avg_drift = monitor.average_timing_drift_ms();
    println!("Average timing drift: {:.1}ms", avg_drift);
    assert!(
        avg_drift < 100.0,
        "Timing drift too high: {:.1}ms",
        avg_drift
    );
}

#[tokio::test]
async fn test_streaming_mixed_with_unary_requests() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Create clients for both streaming and unary requests
    let mut streaming_client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    let mut unary_client = server
        .grpc_client()
        .await
        .expect("Failed to create unary client");

    // Start streaming
    let mut stream = streaming_client
        .start_time_stream()
        .await
        .expect("Failed to start time stream");

    // Interleave streaming and unary requests
    for i in 0..3 {
        // Get streaming message
        let stream_message = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(stream_message.is_ok());
        let stream_response = stream_message.unwrap().unwrap().unwrap();

        // Make unary request
        let unary_request =
            tonic::Request::new(hello_world_grpc::services::hello_world::HelloRequest {
                name: format!("TestUser{}", i).to_string(),
            });

        let unary_response = unary_client.say_hello(unary_request).await;
        assert!(unary_response.is_ok());
        let unary_reply = unary_response.unwrap().into_inner();

        // Both should work correctly
        assert!(validate_rfc3339_timestamp(&stream_response.timestamp));
        assert!(unary_reply.message.contains("TestUser"));

        println!(
            "Iteration {}: Stream={}, Unary={}",
            i, stream_response.timestamp, unary_reply.message
        );
    }
}

#[tokio::test]
async fn test_streaming_error_handling_invalid_requests() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let mut client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    // Stream requests should always succeed (no validation on TimeRequest)
    let stream_result = client.start_time_stream().await;
    assert!(stream_result.is_ok());

    let mut stream = stream_result.unwrap();

    // First message should arrive successfully
    let message = timeout(Duration::from_secs(2), stream.next()).await;
    assert!(message.is_ok());
    assert!(message.unwrap().is_some());
}

#[tokio::test]
async fn test_streaming_network_interruption_simulation() {
    init_test_logging();

    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let mut client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to create streaming client");

    let mut stream = client
        .start_time_stream()
        .await
        .expect("Failed to start time stream");

    // Get first message to establish connection
    let first_message = timeout(Duration::from_secs(2), stream.next()).await;
    assert!(first_message.is_ok());

    // Simulate network interruption by dropping the stream abruptly
    drop(stream);

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Try to reconnect (simulating client retry logic)
    let mut new_client = StreamingClient::from_test_server(&server)
        .await
        .expect("Failed to reconnect after network interruption");

    let mut new_stream = new_client
        .start_time_stream()
        .await
        .expect("Failed to restart stream after interruption");

    // Should be able to get messages from new stream
    let recovery_message = timeout(Duration::from_secs(2), new_stream.next()).await;
    assert!(recovery_message.is_ok());
    assert!(recovery_message.unwrap().is_some());
}
