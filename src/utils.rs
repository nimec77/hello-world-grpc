use std::time::Instant;
use tonic::Request;
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
