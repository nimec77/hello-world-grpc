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
}
