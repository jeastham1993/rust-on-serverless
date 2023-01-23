use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::{handlers::price_order_handler, shared_data::ValidatedOrder};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    println!("Init");

    let res = run(service_fn(|request: LambdaEvent<ValidatedOrder>| {
        price_order_handler(request.payload)
    }))
    .await;

    res
}
