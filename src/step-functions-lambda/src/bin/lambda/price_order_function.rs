use std::fmt;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::state_data::{
    Address, PricedOrder, OrderLine, StateResponse, ValidatedOrder,
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

    let res = run(service_fn(|request: LambdaEvent<StateResponse<ValidatedOrder>>| {
        function_handler(request)
    }))
    .await;

    res
}

async fn function_handler(
    evt: LambdaEvent<StateResponse<ValidatedOrder>>,
) -> Result<StateResponse<PricedOrder>, ValidationError> {

    let line_count = &evt.payload.data.order_lines.len();
    
    Ok(StateResponse {
        data: PricedOrder {
            order_number: evt.payload.data.order_number,
            order_lines: evt.payload.data.order_lines,
            address: evt.payload.data.address,
            total_amount: (line_count * 7) as f64
        },
        events: evt.payload.events,
    })
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    errors: Vec<String>,
}

impl ValidationError {
    pub fn new(message: Vec<String>) -> ValidationError {
        ValidationError { errors: message }
    }
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error: {0}", self.to_string())
    }
}

impl std::error::Error for ValidationError {}
