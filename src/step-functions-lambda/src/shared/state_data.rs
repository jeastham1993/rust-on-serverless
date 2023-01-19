use serde::{Deserialize, Serialize};
use chrono::{DateTime, FixedOffset, Utc};

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

#[derive(Deserialize, Serialize)]
pub struct PricedOrder {
    pub order_number: String,
    pub order_lines: Vec<OrderLine>,
    pub address: Address,
    pub total_amount: f64
}

#[derive(Deserialize, Serialize)]
pub struct OrderValidationCompletedEvent {
    pub order_number: String,
}

#[derive(Deserialize, Serialize)]
pub struct Address {
    pub address_line_1: String,
    pub address_line_2: String,
    pub address_line_3: String,
    pub address_line_4: String,
    pub address_line_5: String,
    pub postcode: String,
}

#[derive(Deserialize, Serialize)]
pub struct OrderLine {
    pub product_code: String,
    pub quantity: f64,
}

#[derive(Deserialize, Serialize)]
pub struct Event {
    pub event_name: String,
    pub payload: String,
    pub event_date: String
}

impl Event {
    pub fn new(event_name: String, payload: String) -> Event {
        Event {
            event_name: event_name,
            payload: payload,
            event_date: Utc::now().to_rfc3339()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct StateResponse<T> {
    pub data: T,
    pub events: Vec<Event>,
}
