use url::Url;
use tungstenite::{connect, Message};
use std::{io::{stdin,stdout,Write}};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

fn main() {
    let (mut socket, response) = connect(
        Url::parse("wss://kpxijue727.execute-api.eu-west-1.amazonaws.com/production").unwrap()
    ).expect("Can't connect");

    let mut valid_address_entered = false;

    let mut address_result: Option<AddressDetails> = Option::None;

    let mut order_lines: Vec<OrderLine> = Vec::new();
    order_lines.push(OrderLine{
        product_code: String::from("PROD123"),
        quantity: 7.0
    });

    while valid_address_entered == false {
        let entered_address = requestAddressDetails();

        if entered_address.is_ok() {
            valid_address_entered = true;
            address_result = Some(entered_address.unwrap());
        }
    }

    let process_order_message = PlaceOrder {
        MessageGroupId: String::from("OrderProcessing"),
        data: ProcessOrder { 
            order_lines: order_lines,
            address: address_result.unwrap() 
        }
    };

    let start = Instant::now();

    let serialized_to_send = serde_json::to_string(&process_order_message);
  
    socket.write_message(Message::Text(serialized_to_send.unwrap().into()));
    
    loop {
        let msg = socket.read_message().expect("Error reading message");
        let duration = start.elapsed();
        println!("Response received in {}ms: {}", duration.as_millis(), msg);
    }
}


fn requestAddressDetails() -> Result<AddressDetails, ()> {
    let mut s=String::new();
    let mut address_details = Vec::new();

    print!("Please enter your address with each line seperated by commas: ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }

    let string_parts = s.split(',');

    for part in string_parts {
        address_details.push(part)
    }

    if address_details.len() != 6 {
        println!("Invalid address entered, ensure you enter all 5 lines and the postcdode");

        return Err(());
    }
    else {
        address_details = address_details.clone();
    }

    Ok(AddressDetails {
        address_line_1: address_details[0].to_string(),
        address_line_2: address_details[1].to_string(),
        address_line_3: address_details[2].to_string(),
        address_line_4: address_details[3].to_string(),
        address_line_5: address_details[4].to_string(),
        postcode: address_details[5].to_string(),
    })
}

#[derive(Deserialize, Serialize)]
struct PlaceOrder {
    MessageGroupId: String,
    data: ProcessOrder,
}

#[derive(Deserialize, Serialize)]
struct ProcessOrder {
    pub order_lines: Vec<OrderLine>,
    pub address: AddressDetails,
}

#[derive(Deserialize, Serialize)]
struct AddressDetails {
    pub address_line_1: String,
    pub address_line_2: String,
    pub address_line_3: String,
    pub address_line_4: String,
    pub address_line_5: String,
    pub postcode: String,
}

#[derive(Deserialize, Serialize)]
struct OrderLine {
    pub product_code: String,
    pub quantity: f64,
}