use lambda_http::Body;
use lambda_runtime::{Error, LambdaEvent};
use aws_lambda_events::event::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use serde::{Serialize, Deserialize};
use crate::shared::{data::{Repository}, models::ToDo};

#[derive(Deserialize, Serialize)]
struct CreateProductCommand {
    pub title: String,
    pub is_completed: bool,
    pub owner_id: String
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

    let command: CreateProductCommand = serde_json::from_str(&body).unwrap();

    let res = client.store_todo(&ToDo::new(command.title, command.is_completed, command.owner_id))
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
            tracing::error!(err);
            
            ApiGatewayV2httpResponse{
            body: Some(Body::Text("Internal server error".to_string())),
            status_code: 500,
            ..Default::default()
        }}),
    }
}

pub async fn get_todo(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>
) -> Result<ApiGatewayV2httpResponse, Error> {
    tracing::info!("Received request from API Gateway");

    let path_parameters = request.payload.path_parameters;
    
    tracing::info!(path_parameters = serde_json::to_string(&path_parameters).unwrap(), "Received request from API Gateway");

    let id = match path_parameters.get("id") {
        Some(id) => id,
        None => {
            tracing::error!("Id not found");
            
            return Ok(ApiGatewayV2httpResponse{
                body: Some(Body::Text("Id required".to_string())),
                status_code: 400,
                ..Default::default()
            })
        },
    };

    if id == "" {
        tracing::error!("Id not found");
            
        return Ok(ApiGatewayV2httpResponse{
            body: Some(Body::Text("Id required".to_string())),
            status_code: 400,
            ..Default::default()
        })
    }

    let res = client.get_todo(id).await;

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(ApiGatewayV2httpResponse{
            body: Some(Body::Text(serde_json::to_string(&res.unwrap()).unwrap())),
            status_code: 200,
            ..Default::default()
        }),
        Err(err) => Ok({
            tracing::error!(err);
            
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
    use std::collections::HashMap;
    use lambda_runtime::{LambdaEvent, Context};

    struct MockRepository {
        should_fail: bool
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_todo(&self, _id: &ToDo) -> Result<String, Error> {
            if self.should_fail {
                return Err("Forced failure!")?;
            }
            else {
                Ok("OK".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_success() {
        let client = MockRepository{should_fail: false};

        let request = build_request("test1".to_string(), "hello".to_string());

        let test_context = build_test_context();

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = function_handler(&client, lambda_event)
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
        let response = function_handler(&client, lambda_event)
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
        let response = function_handler(&client, lambda_event)
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
        let response = function_handler(&client, lambda_event)
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