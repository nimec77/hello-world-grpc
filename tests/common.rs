use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Endpoint};
use tonic_health::server::health_reporter;
use tracing::info;

use hello_world_grpc::config::{
    AppConfig, LogFormat, LogLevel, LoggingConfig, ServerConfig, StreamingConfig,
};
use hello_world_grpc::services::hello_world::{
    greeter_client::GreeterClient, greeter_server::GreeterServer, GreeterService, TimeRequest,
    TimeResponse,
};
use hello_world_grpc::utils::{start_health_server, SimpleMetrics};

/// Test server for integration testing
///
/// Manages the lifecycle of a gRPC server instance for testing purposes.
/// Automatically allocates free ports and handles server startup/shutdown.
#[allow(dead_code)]
pub struct TestServer {
    pub grpc_addr: SocketAddr,
    pub health_port: u16,
    _server_handle: JoinHandle<Result<()>>,
    _health_handle: JoinHandle<Result<()>>,
}

impl TestServer {
    /// Start a new test server on available ports
    ///
    /// This creates and starts a full gRPC server with health checks,
    /// similar to the production server but optimized for testing.
    pub async fn start() -> Result<Self> {
        // Find available ports
        let grpc_addr = find_available_address().await?;
        let health_port = find_available_port().await?;

        info!(
            grpc_addr = %grpc_addr,
            health_port = health_port,
            "Starting test server"
        );

        // Create metrics and service instances
        let metrics = SimpleMetrics::new();
        let greeter_service = GreeterService::new(metrics.clone());

        // Setup gRPC health check service
        let (health_reporter, health_service) = health_reporter();
        health_reporter
            .set_serving::<GreeterServer<GreeterService>>()
            .await;

        // Start the gRPC server
        let grpc_addr_clone = grpc_addr;
        let server_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(health_service)
                .add_service(GreeterServer::new(greeter_service))
                .serve(grpc_addr_clone)
                .await
                .context("gRPC server failed")
        });

        // Start the HTTP health server
        let health_handle = tokio::spawn(async move {
            start_health_server(health_port)
                .await
                .context("HTTP health server failed")
        });

        // Wait a bit for servers to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(TestServer {
            grpc_addr,
            health_port,
            _server_handle: server_handle,
            _health_handle: health_handle,
        })
    }

    /// Create a gRPC client connected to this test server
    pub async fn grpc_client(&self) -> Result<GreeterClient<Channel>> {
        let endpoint = Endpoint::from_shared(format!("http://{}", self.grpc_addr))
            .context("Failed to create endpoint")?;

        let channel = endpoint
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await
            .context("Failed to connect to test server")?;

        Ok(GreeterClient::new(channel))
    }

    /// Get the gRPC server address
    pub fn grpc_address(&self) -> SocketAddr {
        self.grpc_addr
    }

    /// Get the health check port
    #[allow(dead_code)]
    pub fn health_port(&self) -> u16 {
        self.health_port
    }

    /// Get the HTTP health check URL
    #[allow(dead_code)]
    pub fn health_url(&self) -> String {
        format!("http://127.0.0.1:{}/health", self.health_port)
    }
}

/// Test configuration with sensible defaults for integration testing
pub fn test_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            grpc_address: "127.0.0.1:50051".to_string(), // Valid address for testing
            health_port: 8081,                           // Valid port for testing
        },
        logging: LoggingConfig {
            level: LogLevel::Info,
            format: LogFormat::Pretty, // Pretty format for test output
        },
        streaming: StreamingConfig {
            interval_seconds: 1,
            max_connections: 100,
            timeout_seconds: 300,
        },
    }
}

/// Find an available TCP address for testing
async fn find_available_address() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("Failed to bind to available address")?;
    let addr = listener
        .local_addr()
        .context("Failed to get local address")?;
    drop(listener);
    Ok(addr)
}

/// Find an available TCP port for testing
async fn find_available_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("Failed to bind to available port")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Initialize test logging with pretty format and reduced verbosity
pub fn init_test_logging() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_test_writer().pretty().with_target(false))
            .init();
    });
}

/// Streaming client wrapper for easier testing
pub struct StreamingClient {
    client: GreeterClient<Channel>,
}

impl StreamingClient {
    /// Create a new streaming client connected to the given address
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let endpoint = Endpoint::from_shared(format!("http://{}", addr))
            .context("Failed to create endpoint")?;

        let channel = endpoint
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await
            .context("Failed to connect to server")?;

        let client = GreeterClient::new(channel);
        Ok(StreamingClient { client })
    }

    /// Create a streaming client from an existing TestServer
    #[allow(dead_code)]
    pub async fn from_test_server(server: &TestServer) -> Result<Self> {
        Self::connect(server.grpc_address()).await
    }

    /// Start a time stream and return the stream handle
    #[allow(dead_code)]
    pub async fn start_time_stream(&mut self) -> Result<tonic::Streaming<TimeResponse>> {
        let request = tonic::Request::new(TimeRequest {});
        let response = self.client.stream_time(request).await?;
        Ok(response.into_inner())
    }

    /// Get the underlying client for other operations (like say_hello)
    #[allow(dead_code)]
    pub fn client(&mut self) -> &mut GreeterClient<Channel> {
        &mut self.client
    }
}

/// Helper for collecting multiple messages from a stream with timeout
#[allow(dead_code)]
pub async fn collect_stream_messages(
    stream: &mut tonic::Streaming<TimeResponse>,
    count: usize,
    timeout_per_message: Duration,
) -> Result<Vec<TimeResponse>> {
    use tokio::time::timeout;
    use tokio_stream::StreamExt;

    let mut messages = Vec::new();
    for i in 0..count {
        match timeout(timeout_per_message, stream.next()).await {
            Ok(Some(Ok(message))) => {
                messages.push(message);
            }
            Ok(Some(Err(e))) => {
                return Err(anyhow::anyhow!("Stream error on message {}: {}", i, e));
            }
            Ok(None) => {
                return Err(anyhow::anyhow!(
                    "Stream ended unexpectedly after {} messages",
                    i
                ));
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for message {} after {}ms",
                    i,
                    timeout_per_message.as_millis()
                ));
            }
        }
    }
    Ok(messages)
}

/// Helper for testing concurrent streaming clients
#[allow(dead_code)]
pub async fn create_concurrent_streaming_clients(
    server_addr: SocketAddr,
    client_count: usize,
) -> Result<Vec<StreamingClient>> {
    let mut clients = Vec::new();

    for i in 0..client_count {
        match StreamingClient::connect(server_addr).await {
            Ok(client) => clients.push(client),
            Err(e) => return Err(anyhow::anyhow!("Failed to create client {}: {}", i, e)),
        }

        // Small delay between client connections to avoid overwhelming the server
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(clients)
}

/// Helper to validate RFC3339 timestamp format
#[allow(dead_code)]
pub fn validate_rfc3339_timestamp(timestamp: &str) -> bool {
    // Basic RFC3339 format validation
    timestamp.len() >= 20
        && timestamp.contains("T")
        && (timestamp.ends_with("Z") || timestamp.contains("+") || timestamp.contains("-"))
}

/// Performance test helper - measures stream throughput and timing
#[allow(dead_code)]
pub struct StreamPerformanceMonitor {
    start_time: std::time::Instant,
    message_count: usize,
    total_timing_drift: Duration,
    last_message_time: Option<std::time::Instant>,
}

#[allow(dead_code)]
impl StreamPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            message_count: 0,
            total_timing_drift: Duration::ZERO,
            last_message_time: None,
        }
    }

    pub fn record_message(&mut self) {
        let now = std::time::Instant::now();
        self.message_count += 1;

        if let Some(last_time) = self.last_message_time {
            // Measure time since last message (should be ~1 second for default interval)
            let actual_interval = now.duration_since(last_time);
            let expected_interval = Duration::from_secs(1);

            let drift = if actual_interval > expected_interval {
                actual_interval - expected_interval
            } else {
                expected_interval - actual_interval
            };

            self.total_timing_drift += drift;
        }

        self.last_message_time = Some(now);
    }

    pub fn messages_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.message_count as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn average_timing_drift_ms(&self) -> f64 {
        if self.message_count > 1 {
            self.total_timing_drift.as_millis() as f64 / (self.message_count as f64 - 1.0)
        } else {
            0.0
        }
    }

    pub fn total_messages(&self) -> usize {
        self.message_count
    }

    pub fn total_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for StreamPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_available_address() {
        let addr = find_available_address().await.unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_ne!(addr.port(), 0);
    }

    #[tokio::test]
    async fn test_find_available_port() {
        let port = find_available_port().await.unwrap();
        assert_ne!(port, 0);
    }

    #[tokio::test]
    async fn test_config_creation() {
        let config = test_config();
        assert!(config.validate().is_ok());
    }
}
