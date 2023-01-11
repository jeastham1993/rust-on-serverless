use aws_sdk_dynamodb::{Client};
use lambda_http::Body;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_lambda_events::{event::apigw::{ApiGatewayV2httpRequest}, apigw::ApiGatewayV2httpResponse};
use todo::{infrastructure::infrastructure::DynamoDbRepository, domain::domain::{Repository}, domain::entities::{ToDoItem, ToDo}};
use std::{env};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);
    let repository: DynamoDbRepository = DynamoDbRepository::new(dynamodb_client, table_name);

    let _res = run(service_fn(|request: LambdaEvent<ApiGatewayV2httpRequest>| {
        create_todo(&repository, request)
    })).await;

    Ok(())
}

pub async fn create_todo(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>
) -> Result<ApiGatewayV2httpResponse, Error> {
    tracing::info!("Received request from API Gateway");

    // Extract body from request
    let body = match request.payload.body {
        Some(id) => id,
        None => {
            tracing::error!("Body not found");
            
            return Ok(ApiGatewayV2httpResponse{
                body: Some(Body::Text("Body required".to_string())),
                status_code: 400,
                ..Default::default()
            })
        },
    };

    if body == "" {
        tracing::error!("Body not found");
            
        return Ok(ApiGatewayV2httpResponse{
            body: Some(Body::Text("Body required".to_string())),
            status_code: 400,
            ..Default::default()
        })
    }

    let to_do_item: ToDoItem = serde_json::from_str::<ToDoItem>(&body).unwrap();

    let res = client
        .store_todo(&ToDo::new(to_do_item.title, todo::domain::entities::IsComplete::INCOMPLETE, "jameseastham".to_string()))
        .await;

    let response = ApiGatewayV2httpResponse{
        body: Some(Body::Text("item saved".to_string())),
        status_code: 200,
        ..Default::default()
    };

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(response),
        Err(err) => Ok({
            tracing::error!("{}", err.to_string());
            
            ApiGatewayV2httpResponse{
            body: Some(Body::Text("Internal server error".to_string())),
            status_code: 500,
            ..Default::default()
        }}),
    }
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use http::{HeaderMap, HeaderValue};
    use todo::domain::domain::RepositoryError;
    use std::collections::HashMap;
    use lambda_runtime::{LambdaEvent, Context};

    struct MockRepository {
        should_fail: bool
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_todo(&self, _id: &ToDo) -> Result<String, RepositoryError> {
            if self.should_fail {
                return Err(RepositoryError::new("Forced failure!".to_string()));
            }
            else {
                Ok("OK".to_string())
            }
        }

        async fn get_todo(&self, id: &String) -> Result<ToDoItem, RepositoryError> {
            return Err(RepositoryError::new("Forced failure!".to_string())); 
        }
    }

    #[tokio::test]
    async fn test_success() {
        let client = MockRepository{should_fail: false};

        let request = build_request("test1".to_string(), "hello".to_string());

        let test_context = build_test_context();

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = create_todo(&client, lambda_event)
            .await
            .unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), Body::Text("item saved".to_string()));
    }

    #[tokio::test]
    async fn test_repository_failure() {
        let client = MockRepository{should_fail: true};

        let request = build_request("test1".to_string(), "hello".to_string());

        let test_context = build_test_context();

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = create_todo(&client, lambda_event)
            .await
            .unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 500);
        assert_eq!(response.body.unwrap(), Body::Text("Internal server error".to_string()));
    }

    #[tokio::test]
    async fn test_empty_id_should_return_400() {
        let client = MockRepository{should_fail: false};

        let request = build_request("".to_string(), "hello".to_string());

        let test_context = build_test_context();

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = create_todo(&client, lambda_event)
            .await
            .unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(response.body.unwrap(), Body::Text("Id required".to_string()));
    }

    #[tokio::test]
    async fn test_empty_body_should_return_400() {
        let client = MockRepository{should_fail: false};

        let request = build_request("test1".to_string(), "".to_string());

        let test_context = build_test_context();

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = create_todo(&client, lambda_event)
            .await
            .unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 400);
        assert_eq!(response.body.unwrap(), Body::Text("Body required".to_string()));
    }

    fn build_request(id: String, body: String) -> ApiGatewayV2httpRequest {
        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), id);

        ApiGatewayV2httpRequest{
            path_parameters: path_parameters,
            body: Some(body),
            ..Default::default()
        }
    }

    fn build_test_context() -> Context {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        Context::try_from(headers).expect("Failure parsing context")
    }
}