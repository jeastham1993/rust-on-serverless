use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::Body;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_lambda_events::event::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use serde::{Deserialize, Serialize};
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
        function_handler(&repository, request)
    })).await;

    Ok(())
}

async fn function_handler(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>
) -> Result<ApiGatewayV2httpResponse, Error> {
    let path_parameters = request.payload.path_parameters;
    let id = match path_parameters.get("id") {
        Some(id) => id,
        None => {
            println!("id not found in path params");
            
            return Ok(ApiGatewayV2httpResponse{
                body: Some(Body::Text("Id required".to_string())),
                status_code: 400,
                ..Default::default()
            })
        },
    };

    // Extract body from request
    let body = match request.payload.body {
        Some(id) => id,
        None => {
            println!("body not found");
            
            return Ok(ApiGatewayV2httpResponse{
                body: Some(Body::Text("Id required".to_string())),
                status_code: 400,
                ..Default::default()
            })
        },
    };

    tracing::info!(id = id, body = body, "Received request from API Gateway");

    let res = client.store_data(&id.to_string(), &body).await;

    let response = ApiGatewayV2httpResponse{
        body: Some(Body::Text("item saved".to_string())),
        status_code: 200,
        ..Default::default()
    };

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(response),
        Err(_) => Ok(ApiGatewayV2httpResponse{
            body: Some(Body::Text("Internal server error".to_string())),
            status_code: 500,
            ..Default::default()
        }),
    }
}

#[derive(Deserialize, Serialize)]
pub struct MessageBody {
    contents: String,
}

pub struct DynamoDbRepository<'a> {
    client: Client,
    table_name: &'a String,
}

impl DynamoDbRepository<'_> {
    fn new(client: Client, table_name: &String) -> DynamoDbRepository{
        return DynamoDbRepository { client: client, table_name: table_name }
    }
}

#[async_trait]
impl Repository for DynamoDbRepository<'_> {
    async fn store_data(&self, id: &String, body: &String) -> Result<String, Error> {

        tracing::info!("Storing record in DynamoDB");

        let res = self.client
            .put_item()
            .table_name(self.table_name)
            .item("id", AttributeValue::S(id.to_string()))
            .item("payload", AttributeValue::S(body.to_string()))
            .send()
            .await;

        match res {
            Ok(_) => Ok("OK".to_string()),
            Err(e) => return Err(Box::new(e))
        }
    }
}

#[async_trait]
pub trait Repository {
    async fn store_data(
        &self,
        id: &String,
        body: &String
    ) -> Result<String, Error>;
}

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};
    use std::collections::HashMap;
    use lambda_runtime::{LambdaEvent, Context};

    struct MockRepository {
        should_fail: bool
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_data(&self, _id: &String, _body: &String) -> Result<String, Error> {
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

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), "1".to_string());

        let request = ApiGatewayV2httpRequest{
            path_parameters: path_parameters,
            body: Some("hello".to_string()),
            ..Default::default()
        };

        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        let test_context = Context::try_from(headers).expect("Failure parsing context");

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

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), "1".to_string());

        let request = ApiGatewayV2httpRequest{
            path_parameters: path_parameters,
            body: Some("hello".to_string()),
            ..Default::default()
        };

        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        let test_context = Context::try_from(headers).expect("Failure parsing context");

        let lambda_event = LambdaEvent { context: test_context, payload: request };

        // Send mock request to Lambda handler function
        let response = function_handler(&client, lambda_event)
            .await
            .unwrap();

        // Assert that the response is correct
        assert_eq!(response.status_code, 500);
        assert_eq!(response.body.unwrap(), Body::Text("Internal server error".to_string()));

    }
}