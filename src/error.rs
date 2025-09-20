/// Comprehensive error handling for the gRPC service
///
/// Provides structured error types that map cleanly to gRPC status codes
/// and include proper context for debugging and observability.
use tonic::Status;
use tracing::warn;

/// Application-level errors that can occur during gRPC request processing
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Domain validation errors (user input)
    #[error("Invalid input: {message}")]
    ValidationError { message: String },

    /// Internal service errors (server-side)
    #[error("Internal service error: {message}")]
    InternalError { message: String },

    /// Configuration or startup errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Resource not found errors
    #[error("Resource not found: {message}")]
    NotFoundError { message: String },

    /// Service temporarily unavailable
    #[error("Service unavailable: {message}")]
    UnavailableError { message: String },
}

impl AppError {
    /// Create a validation error with context
    pub fn validation<S: Into<String>>(message: S) -> Self {
        AppError::ValidationError {
            message: message.into(),
        }
    }

    /// Create an internal error with context
    pub fn internal<S: Into<String>>(message: S) -> Self {
        AppError::InternalError {
            message: message.into(),
        }
    }

    /// Create a configuration error with context
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        AppError::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a not found error with context
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        AppError::NotFoundError {
            message: message.into(),
        }
    }

    /// Create an unavailable error with context
    pub fn unavailable<S: Into<String>>(message: S) -> Self {
        AppError::UnavailableError {
            message: message.into(),
        }
    }
}

/// Convert AppError to appropriate gRPC Status codes
impl From<AppError> for Status {
    fn from(error: AppError) -> Self {
        match error {
            AppError::ValidationError { message } => {
                warn!(error_type = "validation", message = %message, "Request validation failed");
                Status::invalid_argument(message)
            }
            AppError::InternalError { message } => {
                warn!(error_type = "internal", message = %message, "Internal service error");
                Status::internal(message)
            }
            AppError::ConfigurationError { message } => {
                warn!(error_type = "configuration", message = %message, "Configuration error");
                Status::internal(format!("Service configuration error: {}", message))
            }
            AppError::NotFoundError { message } => {
                warn!(error_type = "not_found", message = %message, "Resource not found");
                Status::not_found(message)
            }
            AppError::UnavailableError { message } => {
                warn!(error_type = "unavailable", message = %message, "Service unavailable");
                Status::unavailable(message)
            }
        }
    }
}

/// Convenience trait for adding context to Results
pub trait ErrorContext<T> {
    /// Add context to an error, converting to AppError::InternalError
    fn with_context<F>(self, f: F) -> Result<T, AppError>
    where
        F: FnOnce() -> String;

    /// Add context to validation errors
    fn with_validation_context<F>(self, f: F) -> Result<T, AppError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context<F>(self, f: F) -> Result<T, AppError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| AppError::internal(format!("{}: {}", f(), e)))
    }

    fn with_validation_context<F>(self, f: F) -> Result<T, AppError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| AppError::validation(format!("{}: {}", f(), e)))
    }
}

/// Convenience type alias for AppError results
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn test_validation_error_to_status() {
        let error = AppError::validation("Invalid input data");
        let status = Status::from(error);

        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("Invalid input data"));
    }

    #[test]
    fn test_internal_error_to_status() {
        let error = AppError::internal("Database connection failed");
        let status = Status::from(error);

        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Database connection failed"));
    }

    #[test]
    fn test_not_found_error_to_status() {
        let error = AppError::not_found("User not found");
        let status = Status::from(error);

        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("User not found"));
    }

    #[test]
    fn test_unavailable_error_to_status() {
        let error = AppError::unavailable("Service temporarily down");
        let status = Status::from(error);

        assert_eq!(status.code(), Code::Unavailable);
        assert!(status.message().contains("Service temporarily down"));
    }

    #[test]
    fn test_configuration_error_to_status() {
        let error = AppError::configuration("Invalid config setting");
        let status = Status::from(error);

        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Service configuration error"));
        assert!(status.message().contains("Invalid config setting"));
    }

    #[test]
    fn test_error_context_trait() {
        let result: Result<i32, &str> = Err("original error");
        let app_result = result.with_context(|| "Failed to process data".to_string());

        assert!(app_result.is_err());
        let error = app_result.unwrap_err();
        assert!(matches!(error, AppError::InternalError { .. }));
        assert!(error.to_string().contains("Failed to process data"));
        assert!(error.to_string().contains("original error"));
    }

    #[test]
    fn test_validation_context_trait() {
        let result: Result<i32, &str> = Err("validation failed");
        let app_result = result.with_validation_context(|| "User input invalid".to_string());

        assert!(app_result.is_err());
        let error = app_result.unwrap_err();
        assert!(matches!(error, AppError::ValidationError { .. }));
        assert!(error.to_string().contains("User input invalid"));
        assert!(error.to_string().contains("validation failed"));
    }
}
