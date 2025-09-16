use anyhow::Result;
use std::time::Duration;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use hello_world_grpc::services::hello_world::{greeter_server::GreeterServer, GreeterService};
use hello_world_grpc::utils::{start_health_server, SimpleMetrics};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    init_logging()?;

    info!("Starting Hello World gRPC Server");

    // Create metrics collection instance
    let metrics = SimpleMetrics::new();

    // Create the gRPC service instance with metrics
    let greeter_service = GreeterService::new(metrics.clone());
    
    // Setup gRPC health check service
    let (health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<GreeterServer<GreeterService>>()
        .await;
    
    // Configure the server address
    let addr = "127.0.0.1:50051".parse()?;
    let health_port = 8081;
    info!("gRPC server listening on {}", addr);
    info!("HTTP health check server will start on port {}", health_port);

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

    // Build and start the gRPC server with health service
    Server::builder()
        .add_service(health_service)
        .add_service(GreeterServer::new(greeter_service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Initialize structured logging with environment-based configuration
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| anyhow::anyhow!("Failed to initialize log filter: {}", e))?;

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(filter)
        .init();

    Ok(())
}
