use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::state_data::{
    PricedOrder, StateResponse, ValidatedOrder,
};

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
        function_handler(request)
    }))
    .await;

    res
}

async fn function_handler(
    evt: LambdaEvent<ValidatedOrder>,
) -> Result<StateResponse<PricedOrder>, Error> {

    let line_count = &evt.payload.order_lines.len();
    
    Ok(StateResponse {
        data: PricedOrder {
            order_number: evt.payload.order_number,
            order_lines: evt.payload.order_lines,
            address: evt.payload.address,
            total_amount: (line_count * 7) as f64
        },
        events: Vec::new()
    })
}