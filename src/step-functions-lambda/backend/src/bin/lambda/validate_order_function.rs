use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::{handlers::validate_order_handler, shared_data::{ProcessOrder, StateResponse, InvalidOrder, ValidatedOrder}};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    println!("Init");

    let res = run(service_fn(|request: LambdaEvent<StateResponse<ProcessOrder>>| {
        validate_order_mapper(request.payload)
    }))
    .await;

    res
}

async fn validate_order_mapper(request: StateResponse<ProcessOrder>) -> Result<StateResponse<ValidatedOrder>, String> {
    let res = validate_order_handler(request.data)
        .await;

        match res {
            Ok(valid_order) => Ok(StateResponse {
                connectionId: request.connectionId,
                data: valid_order
            }),
            Err(e) => Err(e.failure_reason)
        }
}
