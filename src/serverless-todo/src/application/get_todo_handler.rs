use aws_lambda_events::{apigw::ApiGatewayV2httpResponse, event::apigw::ApiGatewayV2httpRequest};
use lambda_http::{
    http::{HeaderMap, HeaderValue},
    Body, Error,
};
use lambda_runtime::LambdaEvent;

use crate::domain::entities::Repository;

pub async fn get_todo_handler(
    client: &dyn Repository,
    request: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    tracing::info!("Received request from API Gateway");

    let path_parameters = request.payload.path_parameters;

    tracing::info!(
        path_parameters = serde_json::to_string(&path_parameters).unwrap(),
        "Received request from API Gateway"
    );

    let id = match path_parameters.get("id") {
        Some(id) => id,
        None => {
            tracing::error!("Id not found");

            return Ok(ApiGatewayV2httpResponse {
                body: Some(Body::Text(format_error_response("Id required".to_string()))),
                status_code: 400,
                headers: default_headers(),
                ..Default::default()
            });
        }
    };

    if id == "" {
        tracing::error!("Id not found");

        return Ok(ApiGatewayV2httpResponse {
            body: Some(Body::Text(format_error_response("Id required".to_string()))),
            status_code: 400,
            headers: default_headers(),
            ..Default::default()
        });
    }

    // Temporarily pull owner from header before implementing authentication/authorization
    let owner = match request.payload.headers.get("Owner") {
        None => "".to_string(),
        Some(val) => val.to_str().unwrap().to_string(),
    };

    let res = client.get_todo(&owner, id).await;

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(ApiGatewayV2httpResponse {
            body: Some(Body::Text(
                serde_json::to_string(&res.unwrap().into_dto()).unwrap(),
            )),
            status_code: 200,
            headers: default_headers(),
            ..Default::default()
        }),
        Err(err) => Ok({
            tracing::error!("{}", err.to_string());

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
