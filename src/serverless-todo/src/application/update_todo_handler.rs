use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use lambda_http::{
    http::{HeaderMap, HeaderValue},
    Body, Error,
};
use lambda_runtime::LambdaEvent;

use crate::domain::{entities::Repository, public_types::UpdateToDoCommand, todo_service};

pub async fn update_todo_handler<T: Repository>(
    client: &T,
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

    // Deserialize the input command
    let to_do_item = serde_json::from_str::<UpdateToDoCommand>(&body.unwrap()).unwrap();

    // Use the service to create a new todo
    // From here we are in pure domain language
    let updated_todo = todo_service::update_todo(to_do_item, client).await;

    // Convert the domain response back to a valid HTTP response
    Ok(ApiGatewayV2httpResponse {
        body: match &updated_todo {
            Ok(val) => Some(Body::Text(serde_json::to_string_pretty(val).unwrap())),
            Err(e) => Some(Body::Text(format_error_response(
                e.to_string(),
            ))),
        },
        status_code: match &updated_todo {
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

    tracing::info!("body: {}", body);

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
    use crate::application::update_todo_handler::update_todo_handler;
    use crate::domain::entities::{Repository, ToDo, Title, OwnerId, ToDoId};
    use crate::domain::error_types::RepositoryError;
    use async_trait::async_trait;
    use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
    use chrono::{Utc, DateTime};
    use http::{HeaderMap, HeaderValue};
    use lambda_http::Body;
    use lambda_runtime::{Context, LambdaEvent};

    struct MockRepository {
        should_fail: bool,
        to_do_status_to_return: String,
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_todo(&self, _: &ToDo) -> Result<(), RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            } else {
                Ok(())
            }
        }

        async fn get_todo(&self, _: &String, _: &String) -> Result<ToDo, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            Ok(ToDo::parse(
                Title::new("title".to_string()).unwrap(),
                OwnerId::new("owner".to_string()).unwrap(),
                Some(self.to_do_status_to_return.to_string()),
                Some(ToDoId::parse("id".to_string()).unwrap()),
                match self.to_do_status_to_return.as_str() {
                    "COMPLETE" => Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()),
                    _ => Option::None
                }
            )
            .unwrap())
        }

        async fn list_todos(&self, _: &String) -> Result<Vec<ToDo>, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }

            let mut todos: Vec<ToDo> = Vec::new();

            todos.push(
                ToDo::parse(
                    Title::new("title".to_string()).unwrap(),
                    OwnerId::new("owner".to_string()).unwrap(),
                    Some(self.to_do_status_to_return.to_string()),
                    Some(ToDoId::parse("id".to_string()).unwrap()),
                    match self.to_do_status_to_return.as_str() {
                        "COMPLETE" => Some(DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()),
                        _ => Option::None
                    }
                )
                .unwrap(),
            );

            Ok(todos)
        }
    }

    #[tokio::test]
    async fn test_valid_request_should_return_success() {
        let client = MockRepository { should_fail: false, to_do_status_to_return: String::from("INCOMPLETE") };

        let request = build_request("test1".to_string(), Some("the title".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = update_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_repository_error_should_return_error() {
        let client = MockRepository { should_fail: true, to_do_status_to_return: String::from("INCOMPLETE") };

        let request = build_request("test1".to_string(), Some("hello".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = update_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Record not found}".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_body_should_return_400() {
        let client = MockRepository { should_fail: false, to_do_status_to_return: String::from("INCOMPLETE") };

        let request = build_request("test1".to_string(), Option::None);

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = update_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Body cannot be empty}".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_title_should_return_400() {
        let client = MockRepository { should_fail: false, to_do_status_to_return: String::from("INCOMPLETE") };

        let request = build_request("test1".to_string(), Some("".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = update_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Must be between 1 and 50 chars}".to_string())
        );
    }

    #[tokio::test]
    async fn test_long_title_should_return_400() {
        let client = MockRepository { should_fail: false, to_do_status_to_return: String::from("INCOMPLETE") };

        let request = build_request("test1".to_string(), Some("fmiooinfweoifbweiufwiuefwiefbweifbweiufbniweufbweiufbwieufweiufbiwuebfweubfweuifbweifbuwiufbweifbw".to_string()));

        let test_context = build_test_context();

        let lambda_event = LambdaEvent {
            context: test_context,
            payload: request,
        };

        // Send mock request to Lambda handler function
        let response = update_todo_handler(&client, lambda_event).await.unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(
            response.body.unwrap(),
            Body::Text("{\"message\": Must be between 1 and 50 chars}".to_string())
        );
    }

    fn build_request(id: String, title: Option<String>) -> ApiGatewayV2httpRequest {
        let body = match title {
            Some(val) => format!(
                "{{
            \"to_do_id\": \"{}\",
            \"owner_id\": \"jameseastham\",
            \"title\": \"{}\",
            \"set_as_complete\": false
        }}",
                id,
                val
            ),
            None => "".to_string(),
        };

        ApiGatewayV2httpRequest {
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
