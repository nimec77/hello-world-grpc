use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Endpoint};
use tonic_health::server::health_reporter;
use tracing::info;

use hello_world_grpc::config::{AppConfig, LogFormat, LogLevel, LoggingConfig, ServerConfig};
use hello_world_grpc::services::hello_world::{
    greeter_client::GreeterClient, greeter_server::GreeterServer, GreeterService,
};
use hello_world_grpc::utils::{start_health_server, SimpleMetrics};

/// Test server for integration testing
///
/// Manages the lifecycle of a gRPC server instance for testing purposes.
/// Automatically allocates free ports and handles server startup/shutdown.
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
    pub fn health_port(&self) -> u16 {
        self.health_port
    }

    /// Get the HTTP health check URL
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
