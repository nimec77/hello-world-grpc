use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    net::SocketAddr,
    str::FromStr,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
    pub streaming: StreamingConfig,
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

/// Streaming-related configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamingConfig {
    pub interval_seconds: u64,
    pub max_connections: u32,
    pub timeout_seconds: u64,
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
            streaming: StreamingConfig {
                interval_seconds: 1,
                max_connections: 100,
                timeout_seconds: 300,
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

        // Validate streaming configuration
        if self.streaming.interval_seconds == 0 {
            anyhow::bail!("Streaming interval must be > 0 seconds, got: 0");
        }

        if self.streaming.interval_seconds > 3600 {
            anyhow::bail!(
                "Streaming interval too large (max 3600s/1h), got: {}s",
                self.streaming.interval_seconds
            );
        }

        if self.streaming.max_connections == 0 {
            anyhow::bail!("Max connections must be > 0, got: 0");
        }

        if self.streaming.max_connections > 10000 {
            anyhow::bail!(
                "Max connections too large (max 10000), got: {}",
                self.streaming.max_connections
            );
        }

        if self.streaming.timeout_seconds == 0 {
            anyhow::bail!("Timeout must be > 0 seconds, got: 0");
        }

        if self.streaming.timeout_seconds > 86400 {
            anyhow::bail!(
                "Timeout too large (max 86400s/24h), got: {}s",
                self.streaming.timeout_seconds
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
        .add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        )
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

    #[test]
    fn test_config_crate_simple() {
        // Simple test with minimal config to debug the config crate
        use config::{Config, Environment};
        use serde::Deserialize;

        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct SimpleConfig {
            test_value: String,
        }

        // Test different env var patterns
        println!("Testing different environment variable patterns:");

        // Pattern 1: Original approach
        std::env::set_var("APP_TEST_VALUE", "test123");
        let result1 = Config::builder()
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()
            .and_then(|c| c.try_deserialize::<SimpleConfig>());
        println!(
            "Pattern 1 (APP_TEST_VALUE with separator '__'): {:?}",
            result1
        );
        std::env::remove_var("APP_TEST_VALUE");

        // Pattern 2: Without separator but with underscore
        std::env::set_var("APP_test_value", "test123");
        let result2 = Config::builder()
            .add_source(Environment::with_prefix("APP"))
            .build()
            .and_then(|c| c.try_deserialize::<SimpleConfig>());
        println!(
            "Pattern 2 (APP_test_value without separator): {:?}",
            result2
        );
        std::env::remove_var("APP_test_value");

        // Pattern 3: Exactly as shown in docs
        std::env::set_var("APP__test_value", "test123");
        let result3 = Config::builder()
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()
            .and_then(|c| c.try_deserialize::<SimpleConfig>());
        println!(
            "Pattern 3 (APP__test_value with separator '__'): {:?}",
            result3
        );
        std::env::remove_var("APP__test_value");

        // Pattern 4: All caps with underscores
        std::env::set_var("APP_TEST_VALUE", "test123");
        let result4 = Config::builder()
            .add_source(Environment::with_prefix("APP"))
            .build()
            .and_then(|c| c.try_deserialize::<SimpleConfig>());
        println!(
            "Pattern 4 (APP_TEST_VALUE without separator): {:?}",
            result4
        );
        std::env::remove_var("APP_TEST_VALUE");
    }

    #[test]
    fn test_streaming_config_validation() {
        // Valid streaming config should pass
        let mut config = AppConfig {
            server: ServerConfig {
                grpc_address: "127.0.0.1:50051".to_string(),
                health_port: 8081,
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                format: LogFormat::Pretty,
            },
            streaming: StreamingConfig {
                interval_seconds: 1,
                max_connections: 100,
                timeout_seconds: 300,
            },
        };
        assert!(config.validate().is_ok());

        // Test interval validation
        config.streaming.interval_seconds = 0;
        assert!(config.validate().is_err());

        config.streaming.interval_seconds = 3601;
        assert!(config.validate().is_err());

        config.streaming.interval_seconds = 1; // Reset

        // Test max_connections validation
        config.streaming.max_connections = 0;
        assert!(config.validate().is_err());

        config.streaming.max_connections = 10001;
        assert!(config.validate().is_err());

        config.streaming.max_connections = 100; // Reset

        // Test timeout validation
        config.streaming.timeout_seconds = 0;
        assert!(config.validate().is_err());

        config.streaming.timeout_seconds = 86401;
        assert!(config.validate().is_err());

        // Edge cases that should be valid
        config.streaming = StreamingConfig {
            interval_seconds: 1, // minimum
            max_connections: 1,  // minimum
            timeout_seconds: 1,  // minimum
        };
        assert!(config.validate().is_ok());

        config.streaming = StreamingConfig {
            interval_seconds: 3600, // maximum
            max_connections: 10000, // maximum
            timeout_seconds: 86400, // maximum
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_streaming_config_defaults() {
        let config = AppConfig::default();

        // Verify default streaming values
        assert_eq!(config.streaming.interval_seconds, 1);
        assert_eq!(config.streaming.max_connections, 100);
        assert_eq!(config.streaming.timeout_seconds, 300);

        // Defaults should be valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_variable_overrides() {
        // Set environment variables - test multiple patterns
        // Pattern from earlier test that worked: APP__test_value
        std::env::set_var("APP__LOGGING__LEVEL", "debug");
        std::env::set_var("APP__LOGGING__FORMAT", "json");
        std::env::set_var("APP__SERVER__GRPC_ADDRESS", "0.0.0.0:9999");
        std::env::set_var("APP__SERVER__HEALTH_PORT", "9090");
        std::env::set_var("APP__STREAMING__INTERVAL_SECONDS", "5");
        std::env::set_var("APP__STREAMING__MAX_CONNECTIONS", "200");
        std::env::set_var("APP__STREAMING__TIMEOUT_SECONDS", "600");

        // Debug: Print all env vars with APP prefix
        println!("Environment variables with APP prefix:");
        for (key, value) in std::env::vars() {
            if key.starts_with("APP_") {
                println!("  {} = {}", key, value);
            }
        }

        // Test config crate behavior step by step
        println!("Testing config crate with just env vars:");
        let env_only_config = Config::builder()
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()
            .unwrap();

        println!("Raw config from env vars: {:?}", env_only_config);

        let result = load_config();
        assert!(
            result.is_ok(),
            "Config loading should succeed: {:?}",
            result.err()
        );

        let config = result.unwrap();

        println!("Loaded config: {:?}", config);

        // Test environment-only config first
        println!("Testing just environment config:");
        let env_config_result = Config::builder()
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .and_then(|c| c.try_deserialize::<AppConfig>());
        println!("Env-only config result: {:?}", env_config_result);

        // Test simple string values first (should work if env vars are being read at all)
        assert_eq!(
            config.server.grpc_address, "0.0.0.0:9999",
            "gRPC address not overridden"
        );
        assert_eq!(
            config.server.health_port, 9090,
            "Health port not overridden"
        );
        assert_eq!(
            config.logging.level,
            LogLevel::Debug,
            "Log level not overridden"
        );
        assert_eq!(
            config.logging.format,
            LogFormat::Json,
            "Log format not overridden"
        );

        // Test streaming configuration overrides
        assert_eq!(
            config.streaming.interval_seconds, 5,
            "Streaming interval not overridden"
        );
        assert_eq!(
            config.streaming.max_connections, 200,
            "Streaming max connections not overridden"
        );
        assert_eq!(
            config.streaming.timeout_seconds, 600,
            "Streaming timeout not overridden"
        );

        // Clean up
        std::env::remove_var("APP__LOGGING__LEVEL");
        std::env::remove_var("APP__LOGGING__FORMAT");
        std::env::remove_var("APP__SERVER__GRPC_ADDRESS");
        std::env::remove_var("APP__SERVER__HEALTH_PORT");
        std::env::remove_var("APP__STREAMING__INTERVAL_SECONDS");
        std::env::remove_var("APP__STREAMING__MAX_CONNECTIONS");
        std::env::remove_var("APP__STREAMING__TIMEOUT_SECONDS");
    }
}
