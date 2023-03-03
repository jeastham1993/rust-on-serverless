use aws_sdk_dynamodb::{model::AttributeValue, Client};
use jsonschema::JSONSchema;
use lambda_http::{service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    lambda_http::run(service_fn(|request: Request| {
        publish(&dynamodb_client, &table_name, request)
    }))
    .await?;

    Ok(())
}

async fn publish(
    client: &Client,
    table_name: &str,
    request: Request,
) -> Result<Response<Body>, Error> {
    let body = match request.body() {
        Body::Empty => "".to_string(),
        Body::Text(body) => body.clone(),
        Body::Binary(body) => String::from_utf8_lossy(body).to_string(),
    };

    let payload: RegisterSchema = serde_json::from_str(&body).unwrap();

    let schema: Value = serde_json::from_str(&payload.schema).unwrap();

    let _ = JSONSchema::compile(&schema).expect("A valid schema");

    let save_resp = client
        .put_item()
        .table_name(table_name)
        .item(
            "PK",
            AttributeValue::S(payload.source.clone().to_uppercase()),
        )
        .item(
            "SK",
            AttributeValue::S(format!(
                "{}#{}",
                payload.detail_type.clone().to_uppercase(),
                payload.version.clone()
            )),
        )
        .item("schema", AttributeValue::S(payload.schema.clone()))
        .send()
        .await;

    // Return a response to the end-user
    match save_resp {
        Ok(_) => Ok(Response::builder().status(200).body("item saved".into())?),
        Err(_) => Ok(Response::builder()
            .status(500)
            .body("internal error".into())?),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RegisterSchema {
    source: String,
    detail_type: String,
    version: String,
    schema: String,
}
