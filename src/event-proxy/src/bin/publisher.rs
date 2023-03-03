use aws_sdk_dynamodb::{model::AttributeValue, Client};
use aws_sdk_eventbridge::model::PutEventsRequestEntry;
use jsonschema::JSONSchema;
use lambda_http::{service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let event_bus_name = env::var("EVENT_BUS_NAME").expect("EVENT_BUS_NAME must be set");
    let eb_client = aws_sdk_eventbridge::Client::new(&config);
    let dynamodb_client = Client::new(&config);

    lambda_http::run(service_fn(|request: Request| {
        publish(&dynamodb_client, &eb_client, &table_name, &event_bus_name, request)
    }))
    .await?;

    Ok(())
}

/// Put Item Lambda function
///
/// This function will run for every invoke of the Lambda function.
async fn publish(
    client: &Client,
    eb_client: &aws_sdk_eventbridge::Client,
    table_name: &str,
    event_bus_name: &str,
    request: Request,
) -> Result<Response<Body>, Error> {
    let body = match request.body() {
        Body::Empty => "".to_string(),
        Body::Text(body) => body.clone(),
        Body::Binary(body) => String::from_utf8_lossy(body).to_string(),
    };

    let payload: PublishEvent = serde_json::from_str(&body).unwrap();

    let schema_query = client
        .get_item()
        .table_name(table_name)
        .key("PK", AttributeValue::S(payload.source.clone().to_uppercase()))
        .key("SK", AttributeValue::S(format!("{}#{}", payload.detail_type.clone().to_uppercase(), payload.version.clone().to_uppercase())))
        .send()
        .await
        .unwrap();

    let schema_string = schema_query.item().unwrap()["schema"].as_s().unwrap();

    let schema: Value = serde_json::from_str(&schema_string).unwrap();

    let compiled = JSONSchema::compile(&schema).expect("A valid schema");

    let to_validate: Value = serde_json::from_str(&payload.detail).unwrap();

    println!("Validating: {}", to_validate);

    let result = compiled.validate(&to_validate);

    let mut error_strings: Vec<String> = Vec::new();

    if let Err(errors) = result {
        for error in errors {
            error_strings.push(format!("{} - {}", error.instance_path, error));
        }
    }

    if error_strings.len() > 0 {
        let error_response = serde_json::to_string(&error_strings).unwrap();

        return Ok(Response::builder().status(500).body(error_response.into())?);
    }

    let evt = PutEventsRequestEntry::builder()
        .source(payload.source.clone())
        .detail_type(payload.detail_type.clone())
        .detail(payload.detail.clone())
        .event_bus_name(event_bus_name)
        .build();

    let pub_resp = eb_client.put_events()
        .entries(evt)
        .send()
        .await;

    match pub_resp {
        Ok(_) => Ok(Response::builder().status(200).body("Published".into())?),
        Err(_) => Ok(Response::builder().status(500).body("Publish error".into())?),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PublishEvent {
    detail: String,
    detail_type: String,
    source: String,
    version: String,
}