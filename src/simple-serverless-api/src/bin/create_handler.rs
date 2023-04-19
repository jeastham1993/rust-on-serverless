use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use std::env;
use opentelemetry::{
    global,
    sdk::trace as sdktrace,
    trace::{Span, Tracer}, Context, runtime,
};
use opentelemetry_aws::trace::XrayPropagator;
use opentelemetry_otlp::{WithExportConfig, ExportConfig};
use opentelemetry_http::{HeaderExtractor};

fn init_tracer() -> sdktrace::TracerProvider {
    global::set_text_map_propagator(XrayPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            sdktrace::config()
                .with_sampler(sdktrace::Sampler::AlwaysOn)
                .with_id_generator(sdktrace::XrayIdGenerator::default()),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_export_config(ExportConfig::default())
                .with_endpoint("http://localhost:4318/v1/traces"),
        )
        .install_batch(runtime::TokioCurrentThread);

    let provider = tracer.unwrap().provider().unwrap();

    provider
}

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    let provider = init_tracer();

    lambda_http::run(service_fn(|request: Request| {
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        let res = put_item(&dynamodb_client, &table_name, parent_context, request);

        for result in provider.force_flush() {
            if let Err(err) = result {
                println!("Flush error: {}", err.to_string());
            }
        }

        res
    }))
    .await?;

    println!("Shutting down");

    Ok(())
}

/// Put Item Lambda function
///
/// This function will run for every invoke of the Lambda function.
async fn put_item(
    client: &Client,
    table_name: &str,
    ctx: Context,
    request: Request,
) -> Result<Response<Body>, Error> {

    let mut span = global::tracer("lambda-xray").start_with_context("Processing", &ctx);

    // Extract path parameter from request
    let path_parameters = request.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            return Ok(Response::builder()
                .status(400)
                .body("id is required".into())?)
        }
    };

    // Extract body from request
    let body = match request.body() {
        Body::Empty => "".to_string(),
        Body::Text(body) => body.clone(),
        Body::Binary(body) => String::from_utf8_lossy(body).to_string(),
    };

    let mut dynamo_span =
        global::tracer("aws-lambda-xray").start_with_context("Write to DynamoDB", &ctx);

    // Put the item in the DynamoDB table
    let res = client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::S(id.to_string()))
        .item("payload", AttributeValue::S(body))
        .send()
        .await;

    dynamo_span.end();

    span.end();

    // Return a response to the end-user
    match res {
        Ok(_) => Ok(Response::builder().status(200).body("item saved".into())?),
        Err(_) => Ok(Response::builder()
            .status(500)
            .body("internal error".into())?),
    }
}
