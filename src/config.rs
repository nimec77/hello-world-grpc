use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    net::SocketAddr,
    str::FromStr,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
}

impl Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::Pretty => write!(f, "pretty"),
            LogFormat::Json => write!(f, "json"),
        }
    }
}

impl FromStr for LogFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pretty" => Ok(LogFormat::Pretty),
            "json" => Ok(LogFormat::Json),
            _ => Err(anyhow::anyhow!("Invalid log format: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", s)),
        }
    }
}

/// Main application configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

/// Server-related configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub grpc_address: String,
    pub health_port: u16,
}

/// Logging-related configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                grpc_address: "127.0.0.1:50051".to_string(),
                health_port: 8081,
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                format: LogFormat::Pretty,
            },
        }
    }
}

impl AppConfig {
    /// Validate configuration values
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate gRPC address can be parsed
        self.server
            .grpc_address
            .parse::<SocketAddr>()
            .map_err(|e| {
                anyhow::anyhow!("Invalid gRPC address '{}': {}", self.server.grpc_address, e)
            })?;

        // Validate health port is in valid range
        if self.server.health_port < 1024 {
            anyhow::bail!(
                "Health port must be >= 1024, got: {}",
                self.server.health_port
            );
        }

        // Log level validation is now handled by the LogLevel enum

        Ok(())
    }
}

/// Load configuration with layered approach:
/// 1. Start with defaults
/// 2. Override with config file (optional)
/// 3. Override with environment variables
pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        // Start with defaults
        .add_source(Config::try_from(&AppConfig::default())?)
        // Add config file if it exists (optional)
        .add_source(File::with_name("config/settings").required(false))
        // Override with environment variables (APP_SERVER__GRPC_ADDRESS, etc.)
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;

    config.try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_grpc_address() {
        let mut config = AppConfig::default();

        // Valid address should pass
        config.server.grpc_address = "127.0.0.1:50051".to_string();
        assert!(config.validate().is_ok());

        // Invalid address should fail
        config.server.grpc_address = "invalid_address".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_health_port() {
        let mut config = AppConfig::default();

        // Valid port should pass
        config.server.health_port = 8081;
        assert!(config.validate().is_ok());

        // Invalid port should fail
        config.server.health_port = 80;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_log_level() {
        let mut config = AppConfig::default();

        // Valid log levels should pass (enum guarantees validity)
        for level in &[
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            config.logging.level = level.clone();
            assert!(
                config.validate().is_ok(),
                "Log level {} should be valid",
                level
            );
        }

        // No need to test invalid levels since enum prevents invalid values
    }

    #[test]
    fn test_config_validation_log_format() {
        let mut config = AppConfig::default();

        // Valid log formats should pass
        for format in &[LogFormat::Pretty, LogFormat::Json] {
            config.logging.format = format.to_owned();
            assert!(
                config.validate().is_ok(),
                "Log format {} should be valid",
                format
            );
        }
    }

    #[test]
    fn test_log_level_from_str() {
        // Valid log levels
        assert_eq!(LogLevel::Trace, "trace".parse().unwrap());
        assert_eq!(LogLevel::Debug, "debug".parse().unwrap());
        assert_eq!(LogLevel::Info, "info".parse().unwrap());
        assert_eq!(LogLevel::Warn, "warn".parse().unwrap());
        assert_eq!(LogLevel::Error, "error".parse().unwrap());

        // Case insensitive
        assert_eq!(LogLevel::Info, "INFO".parse().unwrap());
        assert_eq!(LogLevel::Error, "Error".parse().unwrap());

        // Invalid level should fail
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "trace");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_load_config_with_defaults() {
        // This test validates that load_config() works with just defaults
        let result = load_config();
        assert!(
            result.is_ok(),
            "Should load config with defaults: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert!(config.validate().is_ok(), "Default config should be valid");
    }
}
