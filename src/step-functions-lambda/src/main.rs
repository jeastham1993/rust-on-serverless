use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct InputMessage {
    pub data: String,
}

#[derive(Deserialize, Serialize)]
struct ResponseMessage {
    pub data: String,
}

#[derive(Deserialize, Serialize)]
struct Event {
    pub event_name: String,
    pub payload: String,
}

#[derive(Deserialize, Serialize)]
struct StateResponse<T> {
    data: T,
    events: Vec<Event>,
}

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    println!("Init");

    let res = run(service_fn(|request: LambdaEvent<InputMessage>| {
        function_handler(request)
    }))
    .await;

    res
}

async fn function_handler(
    evt: LambdaEvent<InputMessage>,
) -> Result<StateResponse<ResponseMessage>, Error> {
    let mut evt_response = Vec::new();

    println!("{}", evt.payload.data);

    evt_response.push(Event {
        event_name: "validated".to_string(),
        payload: "{ \"hello\": \"world\" }".to_string(),
    });

    Ok(StateResponse {
        data: ResponseMessage {
            data: "hello".to_string(),
        },
        events: evt_response,
    })
}
