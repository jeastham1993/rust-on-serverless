use aws_lambda_events::{
    apigw::ApiGatewayV2CustomAuthorizerSimpleResponse,
    event::apigw::ApiGatewayV2CustomAuthorizerV2Request,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{Map, Value, json};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let _res = run(service_fn(
        |request: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>| handle_auth(request),
    ))
    .await;

    Ok(())
}

pub async fn handle_auth(
    request: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse, Error> {
    for cookie in request.payload.cookies {
        tracing::debug!("Cookie {}", cookie);
    }

    // TODO: Implement custom authorization logic.
    Ok(ApiGatewayV2CustomAuthorizerSimpleResponse {
        is_authorized: true,
        context: json!({
            "user": "james",
        }),
    })
}
