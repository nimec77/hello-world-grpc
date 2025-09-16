use tonic::{Request, Response, Status};
use tracing::{info, warn};

use crate::utils::{extract_client_info, RequestTimer};
use crate::{GreetingMessage, PersonName};

// Include the generated protobuf types
tonic::include_proto!("hello_world");

/// gRPC service implementation for the Hello World Greeter service
///
/// Provides domain-validated greeting functionality with structured logging.
#[derive(Debug, Default)]
pub struct GreeterService;

#[tonic::async_trait]
impl greeter_server::Greeter for GreeterService {
    /// Handles SayHello RPC requests with domain validation
    ///
    /// Validates the incoming name, generates a greeting, and returns the response.
    /// All requests are logged with structured data for observability including
    /// request ID and duration tracking.
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> std::result::Result<Response<HelloReply>, Status> {
        // Extract client info and start request timing
        let client_info = extract_client_info(&request);
        let timer = RequestTimer::start(client_info.request_id);

        let hello_request = request.into_inner();
        let request_name = &hello_request.name;

        // Log request start with structured fields
        info!(
            request_id = %client_info.request_id,
            method = "SayHello",
            client_addr = %client_info.addr,
            "Processing greeting request"
        );

        // Domain validation: convert raw request to validated domain type
        let person_name = match PersonName::new(request_name) {
            Ok(name) => name,
            Err(validation_error) => {
                warn!(
                    request_id = %client_info.request_id,
                    method = "SayHello",
                    client_addr = %client_info.addr,
                    error = %validation_error,
                    input = request_name,
                    duration_ms = timer.elapsed_ms(),
                    "Invalid request data"
                );

                return Err(Status::invalid_argument(format!(
                    "Invalid name: {}",
                    validation_error
                )));
            }
        };

        // Business logic: generate greeting using domain logic
        let greeting = GreetingMessage::for_person(&person_name);

        let reply = HelloReply {
            message: greeting.as_str().to_string(),
        };

        // Log successful completion with all context
        info!(
            request_id = %client_info.request_id,
            method = "SayHello",
            client_addr = %client_info.addr,
            name = person_name.as_str(),
            duration_ms = timer.elapsed_ms(),
            "Successfully processed greeting request"
        );

        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::greeter_server::Greeter;
    use super::*;
    use tonic::Request;

    #[tokio::test]
    async fn test_say_hello_valid_request() {
        let service = GreeterService;
        let request = Request::new(HelloRequest {
            name: "Alice".to_string(),
        });

        let response = service.say_hello(request).await.unwrap();
        let reply = response.into_inner();

        assert_eq!(reply.message, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_say_hello_trims_whitespace() {
        let service = GreeterService;
        let request = Request::new(HelloRequest {
            name: "  Bob  ".to_string(),
        });

        let response = service.say_hello(request).await.unwrap();
        let reply = response.into_inner();

        assert_eq!(reply.message, "Hello, Bob!");
    }

    #[tokio::test]
    async fn test_say_hello_empty_name_fails() {
        let service = GreeterService;
        let request = Request::new(HelloRequest {
            name: "".to_string(),
        });

        let result = service.say_hello(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("Person name cannot be empty"));
    }

    #[tokio::test]
    async fn test_say_hello_too_long_name_fails() {
        let service = GreeterService;
        let long_name = "a".repeat(101);
        let request = Request::new(HelloRequest { name: long_name });

        let result = service.say_hello(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("100 characters"));
    }
}
