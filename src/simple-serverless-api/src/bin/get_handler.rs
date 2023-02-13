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
        get_item(&data_access, request)
    }))
    .await?;

    Ok(())
}

/// Get an Item from DynamoDB
///
/// This function will run for every invoke of the Lambda function.
async fn get_item<T: DataAccess>(
    data_access: &T,
    request: Request,
) -> Result<Response<Body>, Error> {
    // Extract path parameter from request
    let path_parameters = request.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => return Ok(Response::builder().status(400).body("id is required".into())?),
    };

    // Put the item in the DynamoDB table
    let res = data_access.get(id.to_string()).await;

    // Return a response to the end-user
    match res {
        Ok(query_result) => {
            Ok(Response::builder().status(200).body(query_result.into())?)
        },
        Err(_) => Ok(Response::builder().status(500).body("internal error".into())?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate;
    use rust_sample::MockDataAccess;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_get_item() {
        // Arrange
        let test_id = String::from("12345");

        let mut mock = MockDataAccess::new();

        mock.expect_get()
            .with(predicate::eq(test_id))
            .times(1)
            .returning(|_| Ok(String::from("sample payload")));

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), vec!["12345".to_string()]);

        let request = http::Request::builder()
            .method("GET")
            .uri("/12345")
            .body(Body::Text("test1".to_string()))
            .unwrap()
            .with_path_parameters(path_parameters);

        let response = get_item(&mock, request)
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), &Body::Text("sample payload".to_string()));
    }

    #[tokio::test]
    async fn test_get_item_with_data_access_error_should_return_500() {
        // Arrange
        let test_id = String::from("12345");

        let mut mock = MockDataAccess::new();

        mock.expect_get()
            .with(predicate::eq(test_id))
            .times(1)
            .returning(|_| Err(()));

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), vec!["12345".to_string()]);

        let request = http::Request::builder()
            .method("GET")
            .uri("/12345")
            .body(Body::Text("test1".to_string()))
            .unwrap()
            .with_path_parameters(path_parameters);

        let response = get_item(&mock, request)
            .await
            .unwrap();

        assert_eq!(response.status(), 500);
        assert_eq!(response.body(), &Body::Text("internal error".to_string()));
    }
}