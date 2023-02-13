use aws_sdk_apigatewaymanagement::{types::Blob, Client, config};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Serialize, Deserialize};
use std::env;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let endpoint_url = format!(
        "https://{api_id}.execute-api.{region}.amazonaws.com/{stage}",
        api_id = &env::var("API_ID").expect("API_ID must be set"),
        region = &env::var("REGION").expect("REGION must be set"),
        stage = &env::var("API_STAGE").expect("API_STAGE must be set")
    );

    let shared_config = aws_config::from_env().load().await;
    let api_management_config = config::Builder::from(&shared_config)
        .endpoint_url(endpoint_url)
        .build();

    let client = Client::from_conf(api_management_config);

    let res = run(service_fn(|request: LambdaEvent<SendWebSocketResponse>| {
        send_web_socket_response(&client, request.payload)
    }))
    .await;

    res
}

async fn send_web_socket_response(
    client: &Client,
    evt: SendWebSocketResponse,
) -> Result<(), aws_sdk_apigatewaymanagement::Error> {
    client
        .post_to_connection()
        .connection_id(evt.connection_id)
        .data(Blob::new(evt.data))
        .send()
        .await?;

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct SendWebSocketResponse {
    connection_id: String,
    data: String,
}