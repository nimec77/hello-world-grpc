use anyhow::Result;
use std::time::Duration;
use tonic::Code;

use hello_world_grpc::services::hello_world::{HelloRequest, HelloReply};

// Import common test utilities
mod common {
    include!("../common.rs");
}

use common::{init_test_logging, TestServer};

/// Integration tests for the Hello World gRPC service
///
/// These tests validate end-to-end functionality by starting a real server
/// and making actual gRPC calls through the network stack.

#[tokio::test]
async fn test_say_hello_integration_valid_request() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    let request = HelloRequest {
        name: "Alice".to_string(),
    };

    let response = client.say_hello(request).await.unwrap();
    let reply = response.into_inner();

    assert_eq!(reply.message, "Hello, Alice!");
}

#[tokio::test]
async fn test_say_hello_integration_trims_whitespace() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    let request = HelloRequest {
        name: "  Bob  ".to_string(),
    };

    let response = client.say_hello(request).await.unwrap();
    let reply = response.into_inner();

    assert_eq!(reply.message, "Hello, Bob!");
}

#[tokio::test]
async fn test_say_hello_integration_empty_name_fails() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    let request = HelloRequest {
        name: "".to_string(),
    };

    let result = client.say_hello(request).await;
    assert!(result.is_err());

    let status = result.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("Person name cannot be empty"));
}

#[tokio::test]
async fn test_say_hello_integration_whitespace_only_fails() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    let request = HelloRequest {
        name: "   \t\n  ".to_string(),
    };

    let result = client.say_hello(request).await;
    assert!(result.is_err());

    let status = result.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("Person name cannot be empty"));
}

#[tokio::test]
async fn test_say_hello_integration_too_long_name_fails() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    // Create a name longer than 100 characters
    let long_name = "a".repeat(101);
    let request = HelloRequest { name: long_name };

    let result = client.say_hello(request).await;
    assert!(result.is_err());

    let status = result.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("100 characters"));
}

#[tokio::test]
async fn test_say_hello_integration_concurrent_requests() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();

    // Create multiple clients and send concurrent requests
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let server_addr = server.grpc_address();
            tokio::spawn(async move {
                let endpoint = format!("http://{}", server_addr);
                let channel = tonic::transport::Channel::from_shared(endpoint)
                    .unwrap()
                    .connect()
                    .await
                    .unwrap();
                
                let mut client = hello_world_grpc::services::hello_world::greeter_client::GreeterClient::new(channel);
                
                let request = HelloRequest {
                    name: format!("User{}", i),
                };

                let response = client.say_hello(request).await.unwrap();
                let reply = response.into_inner();
                
                assert_eq!(reply.message, format!("Hello, User{}!", i));
                i
            })
        })
        .collect();

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_grpc_health_check_integration() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    
    // Create a health check client
    let endpoint = format!("http://{}", server.grpc_address());
    let channel = tonic::transport::Channel::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();
    
    let mut health_client = tonic_health::proto::health_client::HealthClient::new(channel);
    
    // Check the health of our service
    let request = tonic_health::proto::HealthCheckRequest {
        service: "hello_world.Greeter".to_string(),
    };
    
    let response = health_client.check(request).await.unwrap();
    let health_response = response.into_inner();
    
    assert_eq!(
        health_response.status,
        tonic_health::proto::health_check_response::ServingStatus::Serving as i32
    );
}

#[tokio::test]
async fn test_http_health_check_integration() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    
    // Wait a bit for the health server to fully start
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Make HTTP request to health endpoint
    let health_url = server.health_url();
    let client = reqwest::Client::new();
    
    let response = client
        .get(&health_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    
    let body: serde_json::Value = response.json().await.unwrap();
    
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "hello-world-grpc");
    assert!(body["timestamp"].is_string());
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_server_startup_and_binding() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    
    // Verify the server is listening on the expected addresses
    assert_ne!(server.grpc_address().port(), 0);
    assert_ne!(server.health_port(), 0);
    
    // Verify we can create a client connection
    let client_result = server.grpc_client().await;
    assert!(client_result.is_ok());
}

#[tokio::test]
async fn test_request_with_various_character_sets() {
    init_test_logging();
    
    let server = TestServer::start().await.unwrap();
    let mut client = server.grpc_client().await.unwrap();

    // Test various character sets
    let test_cases = vec![
        ("ASCII", "Hello, ASCII!"),
        ("José", "Hello, José!"),
        ("李明", "Hello, 李明!"),
        ("🙂 emoji", "Hello, 🙂 emoji!"),
        ("Müller", "Hello, Müller!"),
    ];

    for (input, expected) in test_cases {
        let request = HelloRequest {
            name: input.to_string(),
        };

        let response = client.say_hello(request).await.unwrap();
        let reply = response.into_inner();

        assert_eq!(reply.message, expected, "Failed for input: {}", input);
    }
}
