use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use std::env;
use opentelemetry::{
    global,
    sdk::trace as sdktrace,
    trace::{Span, Tracer, self},
};
use opentelemetry_aws::trace::XrayPropagator;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_http::HeaderExtractor;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracer() -> sdktrace::Tracer {
    global::set_text_map_propagator(XrayPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .with_trace_config(
            sdktrace::config()
                .with_sampler(sdktrace::Sampler::AlwaysOn)
                .with_id_generator(sdktrace::XrayIdGenerator::default()),
        )
        .install_simple();

    tracer.unwrap()
}

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    let tracer = init_tracer();

    let provider = tracer
        .provider()
        .unwrap();

    // Register the Lambda handler
    //
    // We use a closure to pass the `dynamodb_client` and `table_name` as arguments
    // to the handler function.
    lambda_http::run(service_fn(|request: Request| {
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        let x_amzn_trace_id = request
            .headers()
            .get("x-amzn-trace-id")
            .unwrap()
            .to_str()
            .unwrap();

        tracing::info!("Handling - {}", x_amzn_trace_id);

        let mut span = global::tracer("lambda-xray").start_with_context("hello", &parent_context);
        span.add_event(format!("Handling - {x_amzn_trace_id}"), Vec::new());

        let res = put_item(&dynamodb_client, &table_name, request);

        provider.force_flush();

        res
    }))
    .await?;

    Ok(())
}

/// Put Item Lambda function
///
/// This function will run for every invoke of the Lambda function.
async fn put_item(
    client: &Client,
    table_name: &str,
    request: Request,
) -> Result<Response<Body>, Error> {
    // Extract path parameter from request
    let path_parameters = request.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => return Ok(Response::builder().status(400).body("id is required".into())?),
    };

    // Extract body from request
    let body = match request.body() {
        Body::Empty => "".to_string(),
        Body::Text(body) => body.clone(),
        Body::Binary(body) => String::from_utf8_lossy(body).to_string(),
    };

    // Put the item in the DynamoDB table
    let res = client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::S(id.to_string()))
        .item("payload", AttributeValue::S(body))
        .send()
        .await;

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(Response::builder().status(200).body("item saved".into())?),
        Err(_) => Ok(Response::builder().status(500).body("internal error".into())?),
    }
}