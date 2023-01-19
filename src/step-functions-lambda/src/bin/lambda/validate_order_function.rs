use std::fmt;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use order_processing::shared::state_data::{
    Address, Event, OrderLine, ProcessOrder, StateResponse, ValidatedOrder, OrderValidationCompletedEvent,
};
use uuid::Uuid;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    println!("Init");

    let res = run(service_fn(|request: LambdaEvent<ProcessOrder>| {
        function_handler(request)
    }))
    .await;

    res
}

async fn function_handler(
    evt: LambdaEvent<ProcessOrder>,
) -> Result<StateResponse<ValidatedOrder>, ValidationError> {
    let mut evt_response = Vec::new();

    let address_validation = validate_address_details(&evt.payload.address);
    let order_validation = validate_order_lines(&evt.payload.order_lines);

    if address_validation.is_err() || order_validation.is_err() {
        let mut validation_errors = Vec::new();

        let address_errors = address_validation.err();
        let order_errors = order_validation.err();

        match address_errors {
            Option::Some(error) => validation_errors.extend(error.errors),
            _ => (),
        };

        match order_errors {
            Option::Some(error) => validation_errors.extend(error.errors),
            _ => (),
        };

        return Err(ValidationError::new(validation_errors));
    }

    let order_number = Uuid::new_v4().to_string();

    evt_response.push(Event::new(
        "validated".to_string(),
        serde_json::to_string(&OrderValidationCompletedEvent {
            order_number: order_number.to_string(),
        })
        .unwrap(),
    ));

    let mut order_lines = Vec::new();

    for ele in evt.payload.order_lines {
        order_lines.push(OrderLine {
            product_code: ele.product_code,
            quantity: ele.quantity,
        })
    }

    Ok(StateResponse {
        data: ValidatedOrder {
            order_number: order_number,
            order_lines: order_lines,
            address: evt.payload.address,
        },
        events: evt_response,
    })
}

fn validate_order_lines(order_lines: &Vec<OrderLine>) -> Result<(), ValidationError> {
    let mut validation_errors = Vec::new();
    let mut line_count = 1;

    for line in order_lines {
        if line.product_code.trim().len() == 0 {
            validation_errors.push(format!("Line {line_count}: Valid product code required"));
        }

        if line.quantity > 10.0 {
            validation_errors.push(format!(
                "Line {line_count}: Cannot order more than 10 items"
            ));
        }

        line_count = line_count + 1;
    }

    Ok(())
}

fn validate_address_details(address: &Address) -> Result<(), ValidationError> {
    let mut validation_errors = Vec::new();

    if address.address_line_1.trim().len() == 0 {
        validation_errors.push(String::from("Address line 1 is required"));
    }

    if address.address_line_5.trim().len() == 0 {
        validation_errors.push(String::from("Address line 5 is required"));
    }

    if address.postcode.trim().len() == 0 {
        validation_errors.push(String::from("Postcode is required"));
    }

    if validation_errors.len() > 0 {
        return Err(ValidationError::new(validation_errors));
    } else {
        return Ok(());
    }
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
