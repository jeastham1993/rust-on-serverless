use std::fmt;

use aws_lambda_events::encodings::Error;
use rand::Rng;
use uuid::Uuid;

use super::shared_data::{
    Address, InvalidOrder, OrderLine, PricedLine, PricedOrder, ProcessOrder, StateResponse,
    ValidatedOrder,
};

pub async fn validate_order_handler(
    evt: ProcessOrder,
) -> Result<StateResponse<ValidatedOrder>, InvalidOrder> {
    let validation = validate_input(&evt);

    if validation.is_err() {
        let err = validation.err().unwrap();

        return Err(InvalidOrder {
            order_lines: evt.order_lines,
            address: evt.address,
            failure_reason: err.to_string(),
        });
    }

    let order_number = format!("ORD{}", Uuid::new_v4().to_string()[..5].to_uppercase());

    let mut order_lines = Vec::new();

    for ele in evt.order_lines {
        order_lines.push(OrderLine {
            product_code: ele.product_code,
            quantity: ele.quantity,
        })
    }

    Ok(StateResponse {
        data: ValidatedOrder {
            order_number: order_number,
            order_lines: order_lines,
            address: evt.address,
        },
    })
}

pub async fn price_order_handler(evt: ValidatedOrder) -> Result<StateResponse<PricedOrder>, Error> {
    let mut rng = rand::thread_rng();

    let mut total_value = 0.00;
    let mut priced_lines = Vec::new();

    for line in evt.order_lines {
        let unit_price = (rng.gen_range(10..100) as f64) / 10.0;
        let line_price = unit_price * line.quantity;

        priced_lines.push(PricedLine {
            unit_price: unit_price,
            line_price: line_price,
            product_code: line.product_code,
            quantity: line.quantity,
        });

        total_value = total_value + line_price;
    }

    Ok(StateResponse {
        data: PricedOrder {
            order_number: evt.order_number,
            order_lines: priced_lines,
            address: evt.address,
            total_amount: total_value,
        },
    })
}

fn validate_input(order: &ProcessOrder) -> Result<(), ValidationError> {
    let mut validation_results = Vec::new();

    validation_results.push(validate_address_details(&order.address));
    validation_results.push(validate_order_lines(&order.order_lines));

    let mut errors = Vec::new();

    for result in validation_results {
        match result {
            Err(err) => errors.push(err.to_string()),
            Ok(_) => (),
        }
    }

    if !errors.is_empty() {
        return Err(ValidationError::new(errors));
    }

    Ok(())
}

fn validate_order_lines(order_lines: &Vec<OrderLine>) -> Result<(), ValidationError> {
    let mut validation_errors = Vec::new();
    let mut line_count = 1;

    if order_lines.len() == 0 {
        validation_errors.push(String::from("An order must have at least one line"));
    }

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

    if !validation_errors.is_empty() {
        return Err(ValidationError::new(validation_errors));
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

    pub fn to_string(&self) -> String {
        let mut result = String::from("");

        for err in &self.errors {
            result = format!("{result},{err}");
        }

        Self::rem_first_and_last(&result)
    }

    fn rem_first_and_last(value: &String) -> String {
        let mut chars = value.chars();
        chars.next();
        chars.as_str().to_string()
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

/// Unit tests
///
/// These tests are run using the `cargo test` command.
#[cfg(test)]
mod tests {
    use crate::shared::handlers::{validate_order_handler, price_order_handler};
    use crate::shared::shared_data::{Address, OrderLine, ProcessOrder, ValidatedOrder};

    #[tokio::test]
    async fn valid_order_should_return_success() {
        let test_order = generate_test_input(TestOrderSetup { include_lines: true, empty_product_code: false });

        let res = validate_order_handler(test_order).await;

        assert_eq!(res.is_ok(), true);
        assert_eq!(res.as_ref().unwrap().data.order_lines.len(), 1);
        assert_eq!(res.as_ref().unwrap().data.order_number.len() > 0, true);
    }

    #[tokio::test]
    async fn order_with_no_lines_should_return_validation_failure() {
        let test_order = generate_test_input(TestOrderSetup { include_lines: false, empty_product_code: false });

        let res = validate_order_handler(test_order).await;

        assert_eq!(res.is_err(), true);
        assert_eq!(res.as_ref().err().unwrap().order_lines.len(), 0);
        assert_eq!(res.as_ref().err().unwrap().failure_reason, "An order must have at least one line");
    }

    #[tokio::test]
    async fn order_with_empty_product_code_should_return_validation_failure() {
        let test_order = generate_test_input(TestOrderSetup { include_lines: true, empty_product_code: true });

        let res = validate_order_handler(test_order).await;

        assert_eq!(res.is_err(), true);
        assert_eq!(res.as_ref().err().unwrap().order_lines.len(), 1);
        assert_eq!(res.as_ref().err().unwrap().failure_reason, "Line 1: Valid product code required");
    }

    #[tokio::test]
    async fn chain_functions_should_complete() {
        let test_order = generate_test_input(TestOrderSetup { include_lines: true, empty_product_code: false });

        let res = validate_order_handler(test_order).await;

        assert_eq!(res.is_ok(), true);
        assert_eq!(res.as_ref().unwrap().data.order_lines.len(), 1);
        assert_eq!(res.as_ref().unwrap().data.order_number.len() > 0, true);

        let json_data = serde_json::to_string(&res.as_ref().unwrap().data);

        let input= serde_json::from_str(json_data.unwrap().as_str()).unwrap();

        let price_res = price_order_handler(input).await;

        assert_eq!(price_res.is_ok(), true);
        assert_eq!(price_res.as_ref().unwrap().data.total_amount > 0.0, true);
    }

    fn generate_test_input(test_setup: TestOrderSetup) -> ProcessOrder {
        let mut lines = Vec::new();

        if test_setup.include_lines {
            lines.push(OrderLine {
                product_code: match test_setup.empty_product_code {
                    true => "".to_string(),
                    false => "PROD123".to_string(),
                },
                quantity: 1.0,
            });
        }

        ProcessOrder {
            address: Address {
                address_line_1: "123".to_string(),
                address_line_2: "123".to_string(),
                address_line_3: "123".to_string(),
                address_line_4: "123".to_string(),
                address_line_5: "123".to_string(),
                postcode: "123".to_string(),
            },
            order_lines: lines,
        }
    }

    struct TestOrderSetup {
        include_lines: bool,
        empty_product_code: bool
    }
}
