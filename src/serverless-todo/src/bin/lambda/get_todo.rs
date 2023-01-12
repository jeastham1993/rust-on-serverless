use aws_lambda_events::event::apigw::ApiGatewayV2httpRequest;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;
use todo::{
    application::get_todo_handler::get_todo_handler,
    infrastructure::infrastructure::DynamoDbRepository,
};

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

    let _res = run(service_fn(
        |request: LambdaEvent<ApiGatewayV2httpRequest>| get_todo_handler(&repository, request),
    ))
    .await;

    Ok(())
}
