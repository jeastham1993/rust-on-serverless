use aws_sdk_dynamodb::{Client};
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use rust_sample::{DataAccess, DynamoDbDataAccess};
use std::env;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);
    let data_access = DynamoDbDataAccess::new(dynamodb_client, table_name);

    // Register the Lambda handler
    //
    // We use a closure to pass the `dynamodb_client` and `table_name` as arguments
    // to the handler function.
    lambda_http::run(service_fn(|request: Request| {
        delete_item(&data_access, request)
    }))
    .await?;

    Ok(())
}

/// Get an Item from DynamoDB
///
/// This function will run for every invoke of the Lambda function.
async fn delete_item<T: DataAccess>(
    data_access: &T,
    request: Request,
) -> Result<Response<Body>, Error> {
    // Extract path parameter from request
    let path_parameters = request.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            return Ok(Response::builder()
                .status(400)
                .body("id is required".into())?)
        }
    };

    // Delete the item from DynamoDB
    let res = data_access.delete(id.to_string()).await;

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .body("item deleted".into())?),
        Err(_) => Ok(Response::builder()
            .status(500)
            .body("internal error".into())?),
    }
}
