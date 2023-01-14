use aws_lambda_events::{apigw::ApiGatewayV2httpResponse, event::apigw::ApiGatewayV2httpRequest};
use lambda_http::{
    http::{HeaderMap, HeaderValue},
    Body, Error,
};
use lambda_runtime::LambdaEvent;

use crate::domain::{entities::{Repository, OwnerId}, todo_service};

pub async fn list_todo_handler(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    tracing::info!("Received request from API Gateway");

    // Temporarily pull owner from header before implementing authentication/authorization
    let owner = match request.payload.headers.get("Owner") {
        None => "".to_string(),
        Some(val) => val.to_str().unwrap().to_string()
    };

    let res = todo_service::list_todos(OwnerId::OwnerId(owner), client)
        .await;

    // Return a response to the end-user
    match res {
        Ok(data) => Ok(ApiGatewayV2httpResponse {
            body: Some(Body::Text(serde_json::to_string(&data).unwrap())),
            status_code: 200,
            headers: default_headers(),
            ..Default::default()
        }),
        Err(_) => Ok({
            ApiGatewayV2httpResponse {
                body: Some(Body::Text(format_error_response(
                    "Internal server error".to_string(),
                ))),
                status_code: 500,
                headers: default_headers(),
                ..Default::default()
            }
        }),
    }
}

fn format_error_response(err: String) -> String {
    format!("{{\"message\": {}}}", err.to_string())
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str("application/json");

    headers.insert("Content-Type", header_value.unwrap());

    headers
}
