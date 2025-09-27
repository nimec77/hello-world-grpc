use anyhow::{Context, Result};
use std::time::Duration;
use tokio::signal;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use hello_world_grpc::config::{load_config, LogFormat, LoggingConfig};
use hello_world_grpc::services::hello_world::{greeter_server::GreeterServer, GreeterService};
use hello_world_grpc::utils::{start_health_server, SimpleMetrics};

#[tokio::main]
async fn main() -> Result<()> {
    // Load and validate configuration early
    let config = load_config().context("Failed to load configuration")?;

    config
        .validate()
        .context("Configuration validation failed")?;

    // Initialize logging with config
    init_logging(&config.logging)?;

    info!(
        grpc_address = %config.server.grpc_address,
        health_port = config.server.health_port,
        log_level = %config.logging.level,
        log_format = %config.logging.format,
        streaming_interval_seconds = config.streaming.interval_seconds,
        streaming_max_connections = config.streaming.max_connections,
        streaming_timeout_seconds = config.streaming.timeout_seconds,
        version = env!("CARGO_PKG_VERSION"),
        "Starting Hello World gRPC Server with configuration"
    );

    // Create metrics collection instance
    let metrics = SimpleMetrics::new();

    // Create the gRPC service instance with metrics
    let greeter_service = GreeterService::new(metrics.clone());

    // Setup gRPC health check service
    let (health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<GreeterServer<GreeterService>>()
        .await;

    // Parse server address from configuration
    let addr = config
        .server
        .grpc_address
        .parse()
        .context("Failed to parse gRPC address")?;
    let health_port = config.server.health_port;

    info!(address = %addr, "gRPC server will listen on");
    info!(
        port = health_port,
        "HTTP health check server will start on port"
    );

    // Start periodic metrics logging task
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            metrics_clone.log_summary();
        }
    });

    info!("Started periodic metrics logging (every 60 seconds)");

    // Start HTTP health check server
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_port).await {
            tracing::error!(error = %e, "Failed to start health server");
        }
    });

    info!("Started health check servers (gRPC + HTTP)");

    info!("All services configured, starting gRPC server");

    // Create reflection service for grpcurl support
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(hello_world_grpc::FILE_DESCRIPTOR_SET)
        .build_v1()
        .context("Failed to create reflection service")?;

    info!("gRPC reflection service enabled for service discovery");

    // Build and start the gRPC server with graceful shutdown handling
    let server = Server::builder()
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(GreeterServer::new(greeter_service));

    info!("Starting gRPC server with graceful shutdown support");

    // Create graceful shutdown signal handler
    let shutdown_signal = async {
        // Handle different shutdown signals across platforms
        let sigterm = async {
            #[cfg(unix)]
            {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            }
            #[cfg(not(unix))]
            {
                // On Windows, only Ctrl+C is supported
                std::future::pending::<()>().await;
            }
        };

        let sigint = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        tokio::select! {
            _ = sigterm => {
                info!("Received SIGTERM, initiating graceful shutdown");
            },
            _ = sigint => {
                info!("Received Ctrl+C, initiating graceful shutdown");
            },
        }
    };

    // Start server with graceful shutdown and timeout handling
    let shutdown_timeout = Duration::from_secs(30); // Give 30 seconds for graceful shutdown

    // Wrap shutdown logic with timeout to prevent hanging indefinitely
    let server_task = server.serve_with_shutdown(addr, shutdown_signal);

    info!(
        timeout_seconds = shutdown_timeout.as_secs(),
        "gRPC server starting with graceful shutdown timeout"
    );

    // Race the server against a timeout after receiving shutdown signal
    let shutdown_result = tokio::time::timeout(
        shutdown_timeout + Duration::from_secs(35), // Extra buffer for signal handling
        server_task,
    )
    .await;

    match shutdown_result {
        Ok(Ok(())) => {
            info!("gRPC server shut down gracefully within timeout");
        }
        Ok(Err(e)) => {
            warn!(error = %e, "Server encountered error during graceful shutdown");
            return Err(e.into());
        }
        Err(_) => {
            warn!(
                timeout_seconds = shutdown_timeout.as_secs(),
                "Server graceful shutdown exceeded timeout, forcing termination"
            );
            // Note: tonic's serve_with_shutdown handles this well, but we log the timeout
            return Err(anyhow::anyhow!("Server shutdown timeout exceeded"));
        }
    }

    info!("All services stopped gracefully");
    Ok(())
}

/// Initialize structured logging with configuration-based setup
fn init_logging(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(config.level.to_string()))
        .context("Failed to initialize log filter")?;

    match config.format {
        LogFormat::Json => {
            // Production: JSON format for log aggregation
            let json_layer = fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_ansi(false); // Disable colors for JSON output

            tracing_subscriber::registry()
                .with(env_filter)
                .with(json_layer)
                .init();
        }
        LogFormat::Pretty => {
            // Development: Human-readable format
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(false)
                        .with_thread_ids(false),
                )
                .init();
        }
    }

    Ok(())
}
