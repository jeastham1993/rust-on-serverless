use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::state_data::{
    PricedOrder, StateResponse, ValidatedOrder, PricedLine,
};
use rand::Rng;

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
    let mut rng = rand::thread_rng();

    let mut total_value = 0.00;
    let mut priced_lines = Vec::new();

    for line in evt.payload.order_lines {
        let unit_price = (rng.gen_range(10..100) as f64) / 10.0;
        let line_price = unit_price * line.quantity;

        priced_lines.push(PricedLine{
            unit_price: unit_price,
            line_price: line_price,
            product_code: line.product_code,
            quantity: line.quantity
        });

        total_value = total_value + line_price;
    }
    
    Ok(StateResponse {
        data: PricedOrder {
            order_number: evt.payload.order_number,
            order_lines: priced_lines,
            address: evt.payload.address,
            total_amount: total_value
        },
    })
}