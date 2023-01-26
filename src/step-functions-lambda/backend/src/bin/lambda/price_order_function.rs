use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::{handlers::{price_order_handler, PricingError}, shared_data::{ValidatedOrder, PricedOrder, StateResponse}};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    println!("Init");

    let res = run(service_fn(|request: LambdaEvent<StateResponse<ValidatedOrder>>| {
        price_order_mapper(request.payload)
    }))
    .await;

    res
}
async fn price_order_mapper(request: StateResponse<ValidatedOrder>) -> Result<StateResponse<PricedOrder>, StateResponse<PricingError>> {
    let res = price_order_handler(request.data)
        .await;

        match res {
            Ok(priced_order) => Ok(StateResponse {
                connectionId: request.connectionId,
                data: priced_order
            }),
            Err(e) => Err(StateResponse {
                connectionId: request.connectionId,
                data: e
            })
        }
}
