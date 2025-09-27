// Library exports and module structure
pub mod config;
pub mod error;
pub mod utils;

pub mod services {
    pub mod hello_world;
}

/// File descriptor set for gRPC reflection
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/hello_world_descriptor.bin"));

// Domain models with validation
use crate::error::{AppError, AppResult};

const MAX_NAME_LENGTH: usize = 100;

/// A validated person name for greeting requests
///
/// Ensures the name is non-empty, properly trimmed, and within reasonable length limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonName(String);

impl PersonName {
    /// Creates a new PersonName with validation
    ///
    /// # Validation Rules
    /// - Name must not be empty after trimming
    /// - Name must not exceed 100 characters
    /// - Leading/trailing whitespace is automatically trimmed
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::PersonName;
    ///
    /// let name = PersonName::new("Alice").unwrap();
    /// assert_eq!(name.as_str(), "Alice");
    ///
    /// let trimmed = PersonName::new("  Bob  ").unwrap();
    /// assert_eq!(trimmed.as_str(), "Bob");
    ///
    /// assert!(PersonName::new("").is_err());
    /// assert!(PersonName::new("   ").is_err());
    /// ```
    pub fn new(name: impl AsRef<str>) -> AppResult<Self> {
        let trimmed = name.as_ref().trim();

        if trimmed.is_empty() {
            return Err(AppError::validation("Person name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(AppError::validation(format!(
                "Person name cannot exceed {MAX_NAME_LENGTH} characters, got {}",
                trimmed.len()
            )));
        }

        Ok(PersonName(trimmed.to_string()))
    }

    /// Returns the validated name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A formatted greeting message wrapper
///
/// Encapsulates the business logic for generating greeting messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreetingMessage(String);

impl GreetingMessage {
    /// Creates a greeting message for the given person
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::{PersonName, GreetingMessage};
    ///
    /// let name = PersonName::new("Alice").unwrap();
    /// let greeting = GreetingMessage::for_person(&name);
    /// assert_eq!(greeting.as_str(), "Hello, Alice!");
    /// ```
    pub fn for_person(person: &PersonName) -> Self {
        GreetingMessage(format!("Hello, {}!", person.as_str()))
    }

    /// Returns the greeting message as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

use chrono::{DateTime, Utc};
use std::time::Duration;

const DEFAULT_STREAM_INTERVAL_SECS: u64 = 1;
const MIN_STREAM_INTERVAL_MILLIS: u64 = 100; // Minimum 100ms
const MAX_STREAM_INTERVAL_SECS: u64 = 3600; // Maximum 1 hour

/// A validated streaming interval for time updates
///
/// Ensures streaming intervals are within reasonable bounds to prevent performance issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInterval(Duration);

impl StreamInterval {
    /// Creates a new StreamInterval with validation
    ///
    /// # Validation Rules
    /// - Interval must be at least 100ms (to prevent excessive load)
    /// - Interval must not exceed 1 hour (reasonable upper bound)
    /// - Default interval is 1 second
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::StreamInterval;
    /// use std::time::Duration;
    ///
    /// let interval = StreamInterval::new(Duration::from_secs(1)).unwrap();
    /// assert_eq!(interval.as_duration(), Duration::from_secs(1));
    ///
    /// let fast_interval = StreamInterval::new(Duration::from_millis(500)).unwrap();
    /// assert_eq!(fast_interval.as_duration(), Duration::from_millis(500));
    ///
    /// // Too fast - should fail
    /// assert!(StreamInterval::new(Duration::from_millis(50)).is_err());
    ///
    /// // Too slow - should fail  
    /// assert!(StreamInterval::new(Duration::from_secs(7200)).is_err());
    /// ```
    pub fn new(interval: Duration) -> AppResult<Self> {
        let interval_millis = interval.as_millis() as u64;
        let interval_secs = interval.as_secs();

        if interval_millis < MIN_STREAM_INTERVAL_MILLIS {
            return Err(AppError::validation(format!(
                "Stream interval cannot be less than {}ms, got {}ms",
                MIN_STREAM_INTERVAL_MILLIS, interval_millis
            )));
        }

        if interval_secs > MAX_STREAM_INTERVAL_SECS {
            return Err(AppError::validation(format!(
                "Stream interval cannot exceed {} seconds, got {} seconds",
                MAX_STREAM_INTERVAL_SECS, interval_secs
            )));
        }

        Ok(StreamInterval(interval))
    }

    /// Returns the interval as a Duration
    pub fn as_duration(&self) -> Duration {
        self.0
    }

    /// Returns the interval in milliseconds
    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }
}

impl Default for StreamInterval {
    /// Creates the default streaming interval (1 second)
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::StreamInterval;
    /// use std::time::Duration;
    ///
    /// let default_interval = StreamInterval::default();
    /// assert_eq!(default_interval.as_duration(), Duration::from_secs(1));
    /// ```
    fn default() -> Self {
        // Safe unwrap - default value is always valid
        StreamInterval::new(Duration::from_secs(DEFAULT_STREAM_INTERVAL_SECS))
            .expect("Default stream interval should always be valid")
    }
}

/// A validated timestamp snapshot for streaming responses
///
/// Encapsulates RFC3339 timestamp generation and validation business logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSnapshot(DateTime<Utc>);

impl TimeSnapshot {
    /// Creates a new TimeSnapshot for the current UTC time
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::TimeSnapshot;
    ///
    /// let snapshot = TimeSnapshot::now();
    /// let rfc3339 = snapshot.to_rfc3339();
    /// assert!(rfc3339.contains("T"));
    /// assert!(rfc3339.ends_with("Z"));
    /// ```
    pub fn now() -> Self {
        TimeSnapshot(Utc::now())
    }

    /// Creates a TimeSnapshot from an existing DateTime<Utc>
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::TimeSnapshot;
    /// use chrono::{DateTime, Utc};
    ///
    /// let dt = DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z").unwrap().with_timezone(&Utc);
    /// let snapshot = TimeSnapshot::from_datetime(dt);
    /// assert_eq!(snapshot.to_rfc3339(), "2023-01-01T12:00:00Z");
    /// ```
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        TimeSnapshot(dt)
    }

    /// Parses an RFC3339 timestamp string into a TimeSnapshot
    ///
    /// # Examples
    /// ```
    /// use hello_world_grpc::TimeSnapshot;
    ///
    /// let snapshot = TimeSnapshot::from_rfc3339("2023-01-01T12:00:00Z").unwrap();
    /// assert_eq!(snapshot.to_rfc3339(), "2023-01-01T12:00:00Z");
    ///
    /// assert!(TimeSnapshot::from_rfc3339("invalid-timestamp").is_err());
    /// ```
    pub fn from_rfc3339(rfc3339: &str) -> AppResult<Self> {
        let dt = DateTime::parse_from_rfc3339(rfc3339)
            .map_err(|e| AppError::validation(format!("Invalid RFC3339 timestamp: {}", e)))?
            .with_timezone(&Utc);
        Ok(TimeSnapshot(dt))
    }

    /// Returns the timestamp in RFC3339 format
    ///
    /// This format is suitable for gRPC transport and logging.
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Returns the underlying DateTime<Utc>
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Returns the Unix timestamp in seconds
    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_name_valid() {
        let name = PersonName::new("Alice").unwrap();
        assert_eq!(name.as_str(), "Alice");
    }

    #[test]
    fn test_person_name_trims_whitespace() {
        let name = PersonName::new("  Bob  ").unwrap();
        assert_eq!(name.as_str(), "Bob");
    }

    #[test]
    fn test_person_name_empty_fails() {
        assert!(PersonName::new("").is_err());
        assert!(PersonName::new("   ").is_err());
        assert!(PersonName::new("\n\t  ").is_err());
    }

    #[test]
    fn test_person_name_too_long_fails() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        let result = PersonName::new(&long_name);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains(MAX_NAME_LENGTH.to_string().as_str()));
    }

    #[test]
    fn test_person_name_max_length_ok() {
        let max_name = "a".repeat(MAX_NAME_LENGTH);
        let name = PersonName::new(&max_name).unwrap();
        assert_eq!(name.as_str().len(), MAX_NAME_LENGTH);
    }

    #[test]
    fn test_greeting_message_format() {
        let name = PersonName::new("Alice").unwrap();
        let greeting = GreetingMessage::for_person(&name);
        assert_eq!(greeting.as_str(), "Hello, Alice!");
    }

    #[test]
    fn test_greeting_message_with_trimmed_name() {
        let name = PersonName::new("  Bob  ").unwrap();
        let greeting = GreetingMessage::for_person(&name);
        assert_eq!(greeting.as_str(), "Hello, Bob!");
    }

    // StreamInterval tests
    #[test]
    fn test_stream_interval_valid() {
        let interval = StreamInterval::new(Duration::from_secs(1)).unwrap();
        assert_eq!(interval.as_duration(), Duration::from_secs(1));
        assert_eq!(interval.as_millis(), 1000);
    }

    #[test]
    fn test_stream_interval_default() {
        let default_interval = StreamInterval::default();
        assert_eq!(
            default_interval.as_duration(),
            Duration::from_secs(DEFAULT_STREAM_INTERVAL_SECS)
        );
        assert_eq!(
            default_interval.as_millis(),
            DEFAULT_STREAM_INTERVAL_SECS as u128 * 1000
        );
    }

    #[test]
    fn test_stream_interval_fast_valid() {
        let fast_interval = StreamInterval::new(Duration::from_millis(500)).unwrap();
        assert_eq!(fast_interval.as_duration(), Duration::from_millis(500));
        assert_eq!(fast_interval.as_millis(), 500);
    }

    #[test]
    fn test_stream_interval_boundary_min_valid() {
        let boundary_interval =
            StreamInterval::new(Duration::from_millis(MIN_STREAM_INTERVAL_MILLIS)).unwrap();
        assert_eq!(
            boundary_interval.as_millis(),
            MIN_STREAM_INTERVAL_MILLIS as u128
        );
    }

    #[test]
    fn test_stream_interval_boundary_max_valid() {
        let boundary_interval =
            StreamInterval::new(Duration::from_secs(MAX_STREAM_INTERVAL_SECS)).unwrap();
        assert_eq!(
            boundary_interval.as_duration(),
            Duration::from_secs(MAX_STREAM_INTERVAL_SECS)
        );
    }

    #[test]
    fn test_stream_interval_too_fast_fails() {
        let too_fast = Duration::from_millis(MIN_STREAM_INTERVAL_MILLIS - 1);
        let result = StreamInterval::new(too_fast);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be less than"));
    }

    // TimeSnapshot tests
    #[test]
    fn test_time_snapshot_now() {
        let snapshot = TimeSnapshot::now();
        let rfc3339 = snapshot.to_rfc3339();

        // Verify RFC3339 format characteristics
        assert!(rfc3339.contains("T"));
        assert!(rfc3339.ends_with("Z") || rfc3339.ends_with("+00:00")); // Both UTC formats are valid
        assert!(rfc3339.len() >= 20); // Minimum RFC3339 length

        // Verify it's recent (within last minute)
        let now = Utc::now();
        let diff = now.timestamp() - snapshot.timestamp();
        assert!(
            (0..60).contains(&diff),
            "Snapshot should be within last minute"
        );
    }

    #[test]
    fn test_time_snapshot_from_datetime() {
        let dt = DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let snapshot = TimeSnapshot::from_datetime(dt);

        let rfc3339 = snapshot.to_rfc3339();
        assert!(rfc3339 == "2023-01-01T12:00:00Z" || rfc3339 == "2023-01-01T12:00:00+00:00"); // Both UTC formats are valid
        assert_eq!(snapshot.timestamp(), 1672574400); // Unix timestamp for 2023-01-01T12:00:00Z
        assert_eq!(snapshot.as_datetime(), &dt);
    }

    #[test]
    fn test_time_snapshot_from_rfc3339_valid() {
        let rfc3339_input = "2023-01-01T12:00:00Z";
        let snapshot = TimeSnapshot::from_rfc3339(rfc3339_input).unwrap();

        let rfc3339 = snapshot.to_rfc3339();
        assert!(rfc3339 == "2023-01-01T12:00:00Z" || rfc3339 == "2023-01-01T12:00:00+00:00"); // Both UTC formats are valid
        assert_eq!(snapshot.timestamp(), 1672574400);
    }

    #[test]
    fn test_time_snapshot_from_rfc3339_with_offset() {
        let rfc3339_input = "2023-01-01T12:00:00+05:00";
        let snapshot = TimeSnapshot::from_rfc3339(rfc3339_input).unwrap();

        // Should be converted to UTC
        let rfc3339 = snapshot.to_rfc3339();
        assert!(rfc3339 == "2023-01-01T07:00:00Z" || rfc3339 == "2023-01-01T07:00:00+00:00"); // Both UTC formats are valid
        assert_eq!(snapshot.timestamp(), 1672556400); // 5 hours earlier in UTC
    }

    #[test]
    fn test_time_snapshot_from_rfc3339_invalid() {
        let invalid_inputs = vec![
            "",
            "invalid-timestamp",
            "2023-01-01",
            "12:00:00Z",
            "2023-13-01T12:00:00Z", // Invalid month
            "2023-01-32T12:00:00Z", // Invalid day
            "2023-01-01T25:00:00Z", // Invalid hour
        ];

        for invalid_input in invalid_inputs {
            let result = TimeSnapshot::from_rfc3339(invalid_input);
            assert!(
                result.is_err(),
                "Should fail for invalid input: {}",
                invalid_input
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Invalid RFC3339 timestamp"));
        }
    }

    #[test]
    fn test_time_snapshot_rfc3339_roundtrip() {
        let original_rfc3339 = "2023-06-15T14:30:45.123Z";
        let snapshot = TimeSnapshot::from_rfc3339(original_rfc3339).unwrap();
        let roundtrip_rfc3339 = snapshot.to_rfc3339();

        // Should preserve the timestamp (though format might differ slightly)
        let original_snapshot = TimeSnapshot::from_rfc3339(&roundtrip_rfc3339).unwrap();
        assert_eq!(snapshot.timestamp(), original_snapshot.timestamp());
    }

    #[test]
    fn test_time_snapshot_comparison() {
        let snapshot1 = TimeSnapshot::from_rfc3339("2023-01-01T12:00:00Z").unwrap();
        let snapshot2 = TimeSnapshot::from_rfc3339("2023-01-01T12:00:00Z").unwrap();
        let snapshot3 = TimeSnapshot::from_rfc3339("2023-01-01T12:00:01Z").unwrap();

        assert_eq!(snapshot1, snapshot2);
        assert_ne!(snapshot1, snapshot3);
    }
}
