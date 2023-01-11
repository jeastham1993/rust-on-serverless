use aws_sdk_dynamodb::{Client};
use lambda_http::Body;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_lambda_events::{event::apigw::{ApiGatewayV2httpRequest}, apigw::ApiGatewayV2httpResponse};
use todo::{infrastructure::infrastructure::DynamoDbRepository, domain::domain::Repository};
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
        get_todo(&repository, request)
    })).await;

    Ok(())
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
            tracing::error!("{}", err.to_string());
          
            ApiGatewayV2httpResponse{
            body: Some(Body::Text("Internal server error".to_string())),
            status_code: 500,
            ..Default::default()
        }}),
    }
}