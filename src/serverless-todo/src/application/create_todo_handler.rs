use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use lambda_http::{
    http::{HeaderMap, HeaderValue},
    Body, Error,
};
use lambda_runtime::LambdaEvent;

use crate::{
    domain::{entities::Repository, public_types::{UnvalidatedToDo, ToDoItem}, create_todo_service},
};

pub async fn create_todo_handler(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    tracing::info!("Received request from API Gateway");

    // Start with a string
    let body = extract_request_body(request);

    if body == Option::None {
        return Ok(ApiGatewayV2httpResponse {
            body: Some(Body::Text(format_error_response(
                "Body cannot be empty".to_string(),
            ))),
            status_code: 400,
            headers: default_headers(),
            ..Default::default()
        });
    }

    // Build an UnvalidatedToDo
    let to_do_item = serde_json::from_str::<UnvalidatedToDo>(&body.unwrap()).unwrap();

    // Use the service to create a new todo
    // From here we are in pure domain language
    let created_todo = create_todo_service::create_to_do(to_do_item, client).await;

    // Convert the domain response back to a valid HTTP response
    Ok(ApiGatewayV2httpResponse {
        body: match &created_todo {
            Ok(val) => Some(Body::Text(serde_json::to_string_pretty(&ToDoItem {
                id: val.id.get_value(),
                is_complete: val.is_complete.to_string(),
                title: val.title.get_value()
            }).unwrap())),
            Err(err) => Some(Body::Text(format_error_response(err.to_string()))),
        },
        status_code: match &created_todo {
            Ok(_) => 200,
            Err(_) => 400,
        },
        headers: default_headers(),
        ..Default::default()
    })
}

fn format_error_response(err: String) -> String {
    format!("{{\"message\": {}}}", err.to_string())
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str("application/json");

    headers.insert("Content-Type", header_value.unwrap());

    headers
}

fn extract_request_body(request: LambdaEvent<ApiGatewayV2httpRequest>) -> Option<String> {
    let body = match request.payload.body {
        Some(id) => id,
        None => {
            tracing::error!("Body not found");

            return Option::None;
        }
    };

    println!("body: {}", body);

    if body.len() == 0 {
        return Option::None;
    }

    Some(body)
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use crate::application::create_todo_handler::create_todo_handler;
    use crate::domain::entities::{IsComplete, OwnerId, Repository, Title, ToDoId};
    use crate::domain::error_types::RepositoryError;
    use crate::domain::public_types::{CreatedToDo, ToDoItem, ValidatedToDo};
    use async_trait::async_trait;
    use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
    use http::{HeaderMap, HeaderValue};
    use lambda_http::Body;
    use lambda_runtime::{Context, LambdaEvent};
    use std::collections::HashMap;

    struct MockRepository {
        should_fail: bool,
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_todo(&self, _id: ValidatedToDo) -> Result<CreatedToDo, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            } else {
                Ok(CreatedToDo {
                    id: ToDoId::new("test".to_string()).unwrap(),
                    title: Title::new("test".to_string()).unwrap(),
                    is_complete: IsComplete::INCOMPLETE,
                    owner_id: OwnerId::new("test".to_string()).unwrap(),
                })
            }
        }

        async fn get_todo(&self, _id: &String) -> Result<ToDoItem, RepositoryError> {
            return Err(RepositoryError::new("Forced failure!".to_string()));
        }
    }

    #[tokio::test]
    async fn test_valid_request_should_return_success_response() {
        let client = MockRepository { should_fail: false };

        let request = build_request("test1".to_string(), Some("the title".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = create_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), Body::Text("{\n  \"id\": \"test\",\n  \"title\": \"test\",\n  \"is_complete\": \"INCOMPLETE\"\n}".to_string()));
    }

    #[tokio::test]
    async fn test_repository_error_should_return_error() {
        let client = MockRepository { should_fail: true };

        let request = build_request("test1".to_string(), Some("hello".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = create_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Validation error: Failure creating ToDo}".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_body_should_return_400() {
        let client = MockRepository { should_fail: false };

        let request = build_request("test1".to_string(), Option::None);

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = create_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Body cannot be empty}".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_title_should_return_400() {
        let client = MockRepository { should_fail: false };

        let request = build_request("test1".to_string(), Some("".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = create_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Validation error:  - Validation error: Must be between 0 and 50 chars}".to_string())
        );
    }

    #[tokio::test]
    async fn test_long_title_should_return_400() {
        let client = MockRepository { should_fail: false };

        let request = build_request("test1".to_string(), Some("fmiooinfweoifbweiufwiuefwiefbweifbweiufbniweufbweiufbwieufweiufbiwuebfweubfweuifbweifbuwiufbweifbw".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = create_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Validation error:  - Validation error: Must be between 0 and 50 chars}".to_string())
        );
    }

    fn build_request(id: String, title: Option<String>) -> ApiGatewayV2httpRequest {
        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), id);

        let body = match title {
            Some(val) => format!(
                "{{
            \"owner_id\": \"jameseastham\",
            \"title\": \"{}\",
            \"is_complete\": false
        }}",
                val
            ),
            None => "".to_string(),
        };

        ApiGatewayV2httpRequest {
            path_parameters: path_parameters,
            body: Some(body),
            ..Default::default()
        }
    }

    fn build_test_context() -> Context {
        let mut headers = HeaderMap::new();
        headers.insert(
            "lambda-runtime-aws-request-id",
            HeaderValue::from_static("my-id"),
        );
        headers.insert(
            "lambda-runtime-deadline-ms",
            HeaderValue::from_static("123"),
        );
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert(
            "lambda-runtime-trace-id",
            HeaderValue::from_static("arn::myarn"),
        );

        Context::try_from(headers).expect("Failure parsing context")
    }
}
