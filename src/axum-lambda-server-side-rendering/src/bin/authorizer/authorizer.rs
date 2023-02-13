use std::env;

use aws_config::SdkConfig;
use aws_lambda_events::{
    apigw::ApiGatewayV2CustomAuthorizerSimpleResponse,
    event::apigw::ApiGatewayV2CustomAuthorizerV2Request,
};
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Map, Value};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config: SdkConfig = aws_config::load_from_env().await;
    let auth_client: Client = Client::new(&config);
    let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let _res = run(service_fn(
        |request: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>| {
            handle_auth(&auth_client, &table_name, request)
        },
    ))
    .await;

    Ok(())
}

pub async fn handle_auth(
    client: &Client,
    table_name: &String,
    request: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse, Error> {
    tracing::info!("Handling auth");

    let mut token: Option<String> = Option::None;

    for cookie in request.payload.cookies {
        tracing::info!("Checking {}", cookie.clone());

        let cookie_parts: Vec<&str> = cookie.split("=").collect();

        tracing::info!("First part is {}", cookie_parts[0]);

        if cookie_parts[0] == "session_token" {
            token = Some(cookie_parts[1].to_string());
        }
    }

    tracing::info!("Validating token {}", token.clone().unwrap());

    let key = AttributeValue::S(format!("SESSION#{}", token.clone().unwrap().to_uppercase()));

    let valid_session = client
        .get_item()
        .table_name(table_name)
        .key(
            "PK",
            key.clone())
        .key(
            "SK",
            key.clone(),
        )
        .send()
        .await;

    let is_authorised = match valid_session {
        Ok(_) => true,
        Err(_) => false,
    };

    tracing::info!("Returning: {}", is_authorised);

    Ok(ApiGatewayV2CustomAuthorizerSimpleResponse {
        is_authorized: is_authorised,
        context: json!({
            "user": "james",
        }),
    })
}
