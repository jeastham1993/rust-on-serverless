use async_trait::async_trait;
use aws_lambda_events::sqs::SqsMessageObj;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use aws_lambda_events::event::sqs::SqsEventObj;
use aws_lambda_events::event::sqs::SqsBatchResponse;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::env;

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
    let repository = DynamoDbRepository::new(dynamodb_client, table_name);

    let res = run(service_fn(|request: LambdaEvent<SqsEventObj<MessageBody>>| {
        function_handler(&repository, request)
    })).await;

    res
}

async fn function_handler(
    client: &dyn Repository,
    sqs_event: LambdaEvent<SqsEventObj<MessageBody>>
) -> Result<SqsBatchResponse, Error> {
    tracing::info!(records = ?sqs_event.payload.records.len(), "Received request from SQS");

    let mut failures: Vec<String> = Vec::new();

    for ele in &sqs_event.payload.records {
        let res = process_message(ele, client).await;

        if res.is_err() {
            failures.push(ele.message_id.clone().expect("message_id not valid"))
        }
    }

    tracing::info!(failure_count = failures.len(), "Finished processing messages.");

    let data = build_batch_failure_response(failures);

    tracing::info!(data);

    let parsed: SqsBatchResponse = serde_json::from_slice(data.as_bytes()).unwrap();

    Ok(parsed)
}

fn build_batch_failure_response(failures: Vec<String>) -> String
{
    tracing::info!("Building batch response");

    if failures.len() == 0 {
        return "{{\"batchItemFailures\": [] }}".to_string();
    }

    let mut failure_json = "".to_string();

    for failure in &failures {
        if failure_json.len() == 0 {
            failure_json = format!("{{\"itemIdentifier\": \"{failure}\"}}");
        }
        else {
            failure_json = format!("{failure_json},{failure}");
        }
    }

    format!("{{\"batchItemFailures\": [{failure_json}] }}")
}

async fn process_message(message: &SqsMessageObj<MessageBody>, client: &dyn Repository) -> Result<(), Error> {
    tracing::info!(message_id = ?message.message_id, "Processing message");

    if message.body.contents == "fail-me" {
        return Err("Failure processing message!")?;
    }

    let res = client.store_data(&message.body).await;

    if res.is_err() {
        println!("DynamoDB error");

        return Err("Failure processing message!")?;
    }

    Ok(())
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
    async fn store_data(&self, body: &MessageBody) -> Result<String, Error> {

        tracing::info!("Storing record in DynamoDB");

        let res = self.client
            .put_item()
            .table_name(self.table_name)
            .item("id", AttributeValue::S("mysqsmessage".to_string()))
            .item("payload", AttributeValue::S(body.contents.to_string()))
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
        body: &MessageBody
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
    use aws_lambda_events::{event::sqs::SqsEventObj, sqs::SqsMessageObj};
    use lambda_runtime::{LambdaEvent, Context};

    struct MockRepository {
        should_fail: bool
    }

    #[async_trait]
    impl Repository for MockRepository {
        async fn store_data(&self, _body: &MessageBody) -> Result<String, Error> {
            if self.should_fail {
                return Err("Forced failure!")?;
            }
            else {
                Ok("OK".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_process_queue() {
        
        let client = MockRepository{should_fail: false};

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), vec!["1".to_string()]);

        let mut messages: Vec<SqsMessageObj<MessageBody>> = Vec::new();
        messages.push(SqsMessageObj{
            message_id: Some("my-message-id".to_string()),
            aws_region: Some("eu-west-1".to_string()),
            event_source: Option::None,
            event_source_arn: Option::None,
            md5_of_body: Option::None,
            md5_of_message_attributes: Option::None,
            message_attributes: HashMap::new(),
            attributes: HashMap::new(),
            receipt_handle: Option::None,
            body: MessageBody{
                contents: "hello world".to_string()
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        let test_context = Context::try_from(headers).expect("Failure parsing context");

        let sqs_event: SqsEventObj<MessageBody> = SqsEventObj{
            records: messages
        };

        let lambda_event = LambdaEvent { context: test_context, payload: sqs_event };

        // Send mock request to Lambda handler function
        let response = function_handler(&client, lambda_event)
            .await;

        // Assert that the response is correct
        assert_eq!(response.is_ok(), true);

        let full_response = response.expect("Response parsed");

        assert_eq!(full_response.batch_item_failures.len(), 0);

    }

    #[tokio::test]
    async fn test_failed_message() {
        let client = MockRepository{should_fail: false};

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), vec!["1".to_string()]);

        let mut messages: Vec<SqsMessageObj<MessageBody>> = Vec::new();
        messages.push(SqsMessageObj{
            message_id: Some("my-message-id".to_string()),
            aws_region: Some("eu-west-1".to_string()),
            event_source: Option::None,
            event_source_arn: Option::None,
            md5_of_body: Option::None,
            md5_of_message_attributes: Option::None,
            message_attributes: HashMap::new(),
            attributes: HashMap::new(),
            receipt_handle: Option::None,
            body: MessageBody{
                contents: "fail-me".to_string()
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        let test_context = Context::try_from(headers).expect("Failure parsing context");

        let sqs_event: SqsEventObj<MessageBody> = SqsEventObj{
            records: messages
        };

        let lambda_event = LambdaEvent { context: test_context, payload: sqs_event };

        // Send mock request to Lambda handler function
        let response = function_handler(&client, lambda_event)
            .await;

        // Assert that the response is correct
        assert_eq!(response.is_ok(), true);

        let full_response = response.expect("Response parsed");

        assert_eq!(full_response.batch_item_failures.len(), 1);

    }

    #[tokio::test]
    async fn test_repository_failure() {
        let client = MockRepository{should_fail: true};

        // Mock API Gateway request
        let mut path_parameters = HashMap::new();
        path_parameters.insert("id".to_string(), vec!["1".to_string()]);

        let mut messages: Vec<SqsMessageObj<MessageBody>> = Vec::new();
        messages.push(SqsMessageObj{
            message_id: Some("my-message-id".to_string()),
            aws_region: Some("eu-west-1".to_string()),
            event_source: Option::None,
            event_source_arn: Option::None,
            md5_of_body: Option::None,
            md5_of_message_attributes: Option::None,
            message_attributes: HashMap::new(),
            attributes: HashMap::new(),
            receipt_handle: Option::None,
            body: MessageBody{
                contents: "fail-me".to_string()
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));

        let test_context = Context::try_from(headers).expect("Failure parsing context");

        let sqs_event: SqsEventObj<MessageBody> = SqsEventObj{
            records: messages
        };

        let lambda_event = LambdaEvent { context: test_context, payload: sqs_event };

        // Send mock request to Lambda handler function
        let response = function_handler(&client, lambda_event)
            .await;

        // Assert that the response is correct
        assert_eq!(response.is_ok(), true);

        let full_response = response.expect("Response parsed");

        assert_eq!(full_response.batch_item_failures.len(), 1);

    }
}