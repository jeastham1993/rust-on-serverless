use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ProcessOrder {
    pub order_lines: Vec<OrderLine>,
    pub address: Address,
}

#[derive(Deserialize, Serialize)]
pub struct ValidatedOrder {
    pub order_number: String,
    pub order_lines: Vec<OrderLine>,
    pub address: Address,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InvalidOrder {
    pub order_lines: Vec<OrderLine>,
    pub address: Address,
    pub failure_reason: String,
}

impl std::fmt::Display for InvalidOrder {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self.failure_reason)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PricedOrder {
    pub order_number: String,
    pub order_lines: Vec<PricedLine>,
    pub address: Address,
    pub total_amount: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PricedLine {
    pub product_code: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_price: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Address {
    pub address_line_1: String,
    pub address_line_2: String,
    pub address_line_3: String,
    pub address_line_4: String,
    pub address_line_5: String,
    pub postcode: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderLine {
    pub product_code: String,
    pub quantity: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StateResponse<T> {
    pub data: T,
    pub connectionId: String
}

impl<T> std::fmt::Display for StateResponse<T>
where
    T: Serialize,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", serde_json::to_string_pretty(&self.data))
    }
}

#[derive(Deserialize, Serialize)]
pub struct Event {
    pub event_name: String,
    pub payload: String,
    pub event_date: String,
}

impl Event {
    pub fn new(event_name: String, payload: String) -> Event {
        Event {
            event_name: event_name,
            payload: payload,
            event_date: Utc::now().to_rfc3339(),
        }
    }
}
