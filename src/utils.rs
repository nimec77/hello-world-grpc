use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tonic::Request;
use tracing::{error, info};
use uuid::Uuid;

/// Client information extracted from gRPC requests
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub addr: String,
    pub request_id: Uuid,
}

/// Extract client information from gRPC request for logging
///
/// Generates a unique request ID and extracts client address for structured logging.
/// If client address is not available, uses "unknown" as fallback.
pub fn extract_client_info<T>(request: &Request<T>) -> ClientInfo {
    let addr = request
        .remote_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let request_id = Uuid::new_v4();

    ClientInfo { addr, request_id }
}

/// Utility for tracking request duration
#[derive(Debug)]
pub struct RequestTimer {
    start_time: Instant,
    request_id: Uuid,
}

impl RequestTimer {
    /// Start timing a request with the given request ID
    pub fn start(request_id: Uuid) -> Self {
        Self {
            start_time: Instant::now(),
            request_id,
        }
    }

    /// Get elapsed time since timer started
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Get the request ID associated with this timer
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

/// Simple metrics collection with atomic counters
///
/// Tracks basic request statistics for observability without external dependencies.
/// Thread-safe using atomic operations for concurrent access.
#[derive(Debug)]
pub struct SimpleMetrics {
    /// Total number of requests received
    pub requests_total: AtomicU64,
    /// Number of successful requests
    pub requests_success: AtomicU64,
    /// Number of failed requests
    pub requests_error: AtomicU64,
    /// Total duration of all requests in milliseconds
    pub total_duration_ms: AtomicU64,
    /// Number of active streaming connections
    pub active_streams: AtomicU64,
    /// Total number of streams started
    pub streams_started: AtomicU64,
    /// Total number of streams completed (includes disconnections)
    pub streams_completed: AtomicU64,
}

impl SimpleMetrics {
    /// Create a new metrics instance
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            active_streams: AtomicU64::new(0),
            streams_started: AtomicU64::new(0),
            streams_completed: AtomicU64::new(0),
        })
    }

    /// Record a new request with its duration
    pub fn record_request(&self, duration_ms: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_error(&self) {
        self.requests_error.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a new streaming connection started
    pub fn record_stream_started(&self) {
        self.streams_started.fetch_add(1, Ordering::Relaxed);
        self.active_streams.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a streaming connection completed (includes disconnections)
    pub fn record_stream_completed(&self) {
        self.streams_completed.fetch_add(1, Ordering::Relaxed);
        self.active_streams.fetch_sub(1, Ordering::Relaxed);
    }

    /// Log current metrics summary
    pub fn log_summary(&self) {
        let total = self.requests_total.load(Ordering::Relaxed);
        let success = self.requests_success.load(Ordering::Relaxed);
        let errors = self.requests_error.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let active_streams = self.active_streams.load(Ordering::Relaxed);
        let streams_started = self.streams_started.load(Ordering::Relaxed);
        let streams_completed = self.streams_completed.load(Ordering::Relaxed);

        let avg_duration = if total > 0 { total_duration / total } else { 0 };

        let success_rate = if total > 0 {
            (success as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        info!(
            requests_total = total,
            requests_success = success,
            requests_error = errors,
            success_rate = success_rate,
            avg_duration_ms = avg_duration,
            active_streams = active_streams,
            streams_started = streams_started,
            streams_completed = streams_completed,
            "Server metrics summary"
        );
    }
}

/// HTTP health check endpoint handler
///
/// Returns JSON health status including service information, timestamp, and version.
/// Designed for load balancers and monitoring systems.
async fn health_handler(
    _req: HyperRequest<hyper::body::Incoming>,
) -> Result<HyperResponse<String>, Infallible> {
    let health_status = serde_json::json!({
        "status": "healthy",
        "service": "hello-world-grpc",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    });

    let response = HyperResponse::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(health_status.to_string())
        .unwrap();

    Ok(response)
}

/// Start HTTP health check server
///
/// Binds to the specified port and serves health check responses.
/// Runs in a separate async task to avoid blocking the main gRPC server.
pub async fn start_health_server(port: u16) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(&addr).await?;

    info!(port = port, "HTTP health check server started");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(health_handler))
                .await
            {
                error!(error = %err, "Error serving connection");
            }
        });
    }
}
