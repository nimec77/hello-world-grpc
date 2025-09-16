use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
    pub level: String,
    pub format: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                grpc_address: "127.0.0.1:50051".to_string(),
                health_port: 8081,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
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

        // Validate log level
        match self.logging.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => anyhow::bail!(
                "Invalid log level '{}', must be one of: trace, debug, info, warn, error",
                self.logging.level
            ),
        }

        // Validate log format
        match self.logging.format.as_str() {
            "pretty" | "json" => {}
            _ => anyhow::bail!(
                "Invalid log format '{}', must be one of: pretty, json",
                self.logging.format
            ),
        }

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

        // Valid log levels should pass
        for level in &["trace", "debug", "info", "warn", "error"] {
            config.logging.level = level.to_string();
            assert!(
                config.validate().is_ok(),
                "Log level {} should be valid",
                level
            );
        }

        // Invalid log level should fail
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_log_format() {
        let mut config = AppConfig::default();

        // Valid log formats should pass
        for format in &["pretty", "json"] {
            config.logging.format = format.to_string();
            assert!(
                config.validate().is_ok(),
                "Log format {} should be valid",
                format
            );
        }

        // Invalid log format should fail
        config.logging.format = "invalid".to_string();
        assert!(config.validate().is_err());
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
