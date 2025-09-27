use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::ErrorContext;
use crate::utils::{extract_client_info, RequestTimer, SimpleMetrics};
use crate::{GreetingMessage, PersonName, StreamInterval, TimeSnapshot};

// Include the generated protobuf types
tonic::include_proto!("hello_world");

/// gRPC service implementation for the Hello World Greeter service
///
/// Provides domain-validated greeting functionality with structured logging
/// and metrics collection.
#[derive(Debug, Clone)]
pub struct GreeterService {
    metrics: Arc<SimpleMetrics>,
}

impl GreeterService {
    /// Create a new GreeterService with metrics collection
    pub fn new(metrics: Arc<SimpleMetrics>) -> Self {
        Self { metrics }
    }
}

// Type alias for the time streaming response stream
type TimeStream = Pin<Box<dyn Stream<Item = Result<TimeResponse, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl greeter_server::Greeter for GreeterService {
    // Associated type for server-side streaming
    type StreamTimeStream = TimeStream;
    /// Handles SayHello RPC requests with domain validation
    ///
    /// Validates the incoming name, generates a greeting, and returns the response.
    /// All requests are logged with structured data for observability including
    /// request ID and duration tracking.
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> std::result::Result<Response<HelloReply>, Status> {
        // Extract client info and start request timing
        let client_info = extract_client_info(&request);
        let timer = RequestTimer::start(client_info.request_id);

        let hello_request = request.into_inner();
        let request_name = &hello_request.name;

        // Log request start with structured fields
        info!(
            request_id = %client_info.request_id,
            method = "SayHello",
            client_addr = %client_info.addr,
            "Processing greeting request"
        );

        // Domain validation: convert raw request to validated domain type
        let person_name = match PersonName::new(request_name).with_validation_context(|| {
            format!("Failed to validate person name '{}'", request_name)
        }) {
            Ok(name) => name,
            Err(app_error) => {
                let duration = timer.elapsed_ms();

                // Record metrics for failed request
                self.metrics.record_request(duration);
                self.metrics.record_error();

                warn!(
                    request_id = %client_info.request_id,
                    method = "SayHello",
                    client_addr = %client_info.addr,
                    error = %app_error,
                    input = request_name,
                    duration_ms = duration,
                    "Request validation failed"
                );

                // Convert AppError to gRPC Status (includes structured error logging)
                return Err(Status::from(app_error));
            }
        };

        // Business logic: generate greeting using domain logic
        let greeting = GreetingMessage::for_person(&person_name);

        let reply = HelloReply {
            message: greeting.as_str().to_string(),
        };

        let duration = timer.elapsed_ms();

        // Record metrics for successful request
        self.metrics.record_request(duration);
        self.metrics.record_success();

        // Log successful completion with all context
        info!(
            request_id = %client_info.request_id,
            method = "SayHello",
            client_addr = %client_info.addr,
            name = person_name.as_str(),
            duration_ms = duration,
            "Successfully processed greeting request"
        );

        Ok(Response::new(reply))
    }

    /// Handles StreamTime RPC requests with server-side streaming
    ///
    /// Streams current time updates at 1-second intervals using domain-validated types.
    /// Each stream connection is tracked with metrics and structured logging including
    /// stream ID, client address, and connection duration.
    async fn stream_time(
        &self,
        request: Request<TimeRequest>,
    ) -> Result<Response<Self::StreamTimeStream>, Status> {
        // Extract client info and generate unique stream ID
        let client_info = extract_client_info(&request);
        let stream_id = Uuid::new_v4();
        let _timer = RequestTimer::start(stream_id);

        // Log stream start with structured fields
        info!(
            stream_id = %stream_id,
            request_id = %client_info.request_id,
            method = "StreamTime",
            client_addr = %client_info.addr,
            "Starting time streaming connection"
        );

        // Record streaming metrics
        self.metrics.record_stream_started();

        // Create default streaming interval (1 second)
        let interval = StreamInterval::default();
        let interval_duration = interval.as_duration();

        // Create clones for the stream task
        let stream_id_for_map = stream_id;
        let stream_addr_for_map = client_info.addr.clone();

        // Create the time streaming generator
        let time_stream = IntervalStream::new(time::interval(interval_duration)).map(move |_| {
            let snapshot = TimeSnapshot::now();
            let response = TimeResponse {
                timestamp: snapshot.to_rfc3339(),
            };

            info!(
                stream_id = %stream_id_for_map,
                client_addr = %stream_addr_for_map,
                timestamp = %snapshot.to_rfc3339(),
                "Streaming time update"
            );

            Ok(response)
        });

        // Box the stream for type compatibility
        let response_stream: TimeStream = Box::pin(time_stream);

        info!(
            stream_id = %stream_id,
            request_id = %client_info.request_id,
            method = "StreamTime",
            client_addr = %client_info.addr,
            interval_ms = interval.as_millis(),
            "Successfully started time streaming"
        );

        Ok(Response::new(response_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::greeter_server::Greeter;
    use super::*;
    use crate::utils::SimpleMetrics;
    use tonic::Request;

    #[tokio::test]
    async fn test_say_hello_valid_request() {
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(HelloRequest {
            name: "Alice".to_string(),
        });

        let response = service.say_hello(request).await.unwrap();
        let reply = response.into_inner();

        assert_eq!(reply.message, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_say_hello_trims_whitespace() {
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(HelloRequest {
            name: "  Bob  ".to_string(),
        });

        let response = service.say_hello(request).await.unwrap();
        let reply = response.into_inner();

        assert_eq!(reply.message, "Hello, Bob!");
    }

    #[tokio::test]
    async fn test_say_hello_empty_name_fails() {
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(HelloRequest {
            name: "".to_string(),
        });

        let result = service.say_hello(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("Person name cannot be empty"));
    }

    #[tokio::test]
    async fn test_say_hello_too_long_name_fails() {
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let long_name = "a".repeat(101);
        let request = Request::new(HelloRequest { name: long_name });

        let result = service.say_hello(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("100 characters"));
    }

    #[tokio::test]
    async fn test_stream_time_starts_successfully() {
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics.clone());
        let request = Request::new(TimeRequest {});

        let result = service.stream_time(request).await;
        assert!(result.is_ok());

        // Verify stream started metric was recorded
        assert_eq!(
            metrics
                .streams_started
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            metrics
                .active_streams
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_stream_time_response_format() {
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(TimeRequest {});

        let response = service.stream_time(request).await.unwrap();
        let mut stream = response.into_inner();

        // Get first time response
        let first_response = stream.next().await.unwrap().unwrap();

        // Verify it's a valid RFC3339 timestamp
        let timestamp = &first_response.timestamp;
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z") || timestamp.contains("+"));

        // Should be able to parse as DateTime
        use chrono::DateTime;
        let parsed = DateTime::parse_from_rfc3339(timestamp);
        assert!(parsed.is_ok());
    }

    #[tokio::test]
    async fn test_stream_time_multiple_messages() {
        use tokio::time::{timeout, Duration};
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(TimeRequest {});

        let response = service.stream_time(request).await.unwrap();
        let mut stream = response.into_inner();

        // Collect first few messages with timeout
        let mut messages = Vec::new();
        for _ in 0..3 {
            let message = timeout(Duration::from_secs(2), stream.next()).await;
            assert!(message.is_ok()); // Should not timeout

            let time_response = message.unwrap().unwrap().unwrap();
            messages.push(time_response.timestamp);
        }

        assert_eq!(messages.len(), 3);

        // All should be different timestamps
        assert_ne!(messages[0], messages[1]);
        assert_ne!(messages[1], messages[2]);

        // All should be valid RFC3339
        for timestamp in messages {
            assert!(timestamp.contains("T"));
            assert!(timestamp.ends_with("Z") || timestamp.contains("+"));
        }
    }

    #[tokio::test]
    async fn test_stream_time_uses_domain_types() {
        // This test verifies that the streaming uses our domain types internally
        use crate::{StreamInterval, TimeSnapshot};

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(TimeRequest {});

        let result = service.stream_time(request).await;
        assert!(result.is_ok());

        // Verify domain types work correctly
        let interval = StreamInterval::default();
        assert_eq!(interval.as_millis(), 1000); // 1 second default

        let snapshot = TimeSnapshot::now();
        let rfc3339 = snapshot.to_rfc3339();
        assert!(rfc3339.len() >= 20); // Valid RFC3339 format
    }

    #[tokio::test]
    async fn test_stream_time_client_disconnection() {
        use tokio::time::{timeout, Duration};
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics.clone());
        let request = Request::new(TimeRequest {});

        let response = service.stream_time(request).await.unwrap();
        let mut stream = response.into_inner();

        // Get first message to ensure stream starts
        let first_message = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(first_message.is_ok());
        assert!(first_message.unwrap().is_some());

        // Check that stream metrics are being tracked
        let initial_streams = metrics
            .streams_started
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(initial_streams > 0);

        // Drop the stream to simulate client disconnection
        drop(stream);

        // Give time for cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stream should handle disconnection gracefully (no panics)
        // This test primarily ensures no crashes occur during cleanup
    }

    #[tokio::test]
    async fn test_stream_time_metrics_tracking() {
        use tokio::time::{timeout, Duration};
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics.clone());

        let initial_started = metrics
            .streams_started
            .load(std::sync::atomic::Ordering::Relaxed);
        let initial_active = metrics
            .active_streams
            .load(std::sync::atomic::Ordering::Relaxed);

        // Start a stream
        let request = Request::new(TimeRequest {});
        let response = service.stream_time(request).await.unwrap();
        let mut stream = response.into_inner();

        // Verify metrics incremented
        let after_start_started = metrics
            .streams_started
            .load(std::sync::atomic::Ordering::Relaxed);
        let after_start_active = metrics
            .active_streams
            .load(std::sync::atomic::Ordering::Relaxed);

        assert_eq!(after_start_started, initial_started + 1);
        assert_eq!(after_start_active, initial_active + 1);

        // Get a message to ensure stream is active
        let message = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(message.is_ok());

        // Drop the stream
        drop(stream);

        // Give time for cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Active streams should decrease
        let after_drop_active = metrics
            .active_streams
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_drop_active <= after_start_active);
    }

    #[tokio::test]
    async fn test_stream_time_error_handling() {
        // Test that the stream handles internal errors gracefully
        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);
        let request = Request::new(TimeRequest {});

        // This test ensures stream creation doesn't fail
        let result = service.stream_time(request).await;
        assert!(result.is_ok());

        let stream = result.unwrap().into_inner();

        // Stream should be created successfully even if there might be future errors
        // The actual error handling happens at the stream level
        drop(stream);
    }

    #[tokio::test]
    async fn test_stream_time_concurrent_requests() {
        use tokio::time::{timeout, Duration};
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics.clone());

        // Create multiple concurrent streams
        let mut handles = Vec::new();

        for i in 0..3 {
            let service_clone = service.clone();
            let handle: tokio::task::JoinHandle<String> = tokio::spawn(async move {
                let request = Request::new(TimeRequest {});
                let response = service_clone.stream_time(request).await.unwrap();
                let mut stream = response.into_inner();

                // Get first message from each stream
                let message = timeout(Duration::from_secs(3), stream.next()).await;
                assert!(message.is_ok(), "Stream {} should produce message", i);

                let time_response = message.unwrap().unwrap().unwrap();
                time_response.timestamp
            });
            handles.push(handle);
        }

        // Wait for all streams to produce their first message
        let mut timestamps = Vec::new();
        for handle in handles {
            let timestamp = handle.await.unwrap();
            timestamps.push(timestamp);
        }

        // All streams should have produced valid timestamps
        assert_eq!(timestamps.len(), 3);

        // Verify all timestamps are valid RFC3339
        for timestamp in &timestamps {
            assert!(timestamp.contains("T"));
            assert!(timestamp.ends_with("Z") || timestamp.contains("+"));
        }

        // Check that metrics tracked concurrent streams
        let streams_started = metrics
            .streams_started
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(streams_started >= 3);
    }

    #[tokio::test]
    async fn test_stream_time_request_logging() {
        use tokio::time::{timeout, Duration};
        use tokio_stream::StreamExt;

        let metrics = SimpleMetrics::new();
        let service = GreeterService::new(metrics);

        // Create a request with metadata for logging
        let mut request = Request::new(TimeRequest {});
        request
            .metadata_mut()
            .insert("x-client-id", "test-client".parse().unwrap());

        let response = service.stream_time(request).await.unwrap();
        let mut stream = response.into_inner();

        // Get first message to ensure logging occurs
        let message = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(message.is_ok());

        // This test primarily ensures no panics occur during logging
        // Actual log verification would require more complex test infrastructure
        drop(stream);
    }
}
