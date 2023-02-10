use std::env;

use aws_config::SdkConfig;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
    extract::State,
};
use serde::Serialize;
use std::sync::Arc;

use lambda_http::{run, Error, aws_lambda_events::serde::Deserialize};
use serde_json::{json, Value};

struct AppState {
    dynamodb_client: Client,
    table_name: String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let config: SdkConfig = aws_config::load_from_env().await;
    let dynamodb_client: Client = Client::new(&config);
    let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let shared_state = Arc::new(AppState { 
        dynamodb_client: dynamodb_client,
        table_name: table_name.clone()
     });

    let app = Router::new()
        .route("/", get(root))
        .route("/todo", get(list_todo).post(post_todo))
        .route("/todo/:id", get(get_todo))
        .with_state(shared_state);

    run(app).await
}

async fn root() -> Json<Value> {
    Json(json!({ "msg": "Welcome to the Rust ToDo API" }))
}

async fn list_todo(State(state): State<Arc<AppState>>) -> Json<Value> {
    let res = state.dynamodb_client
        .query()
        .table_name(&state.table_name)
        .key_condition_expression("PK = :hashKey")
        .expression_attribute_values(":hashKey", AttributeValue::S(String::from("USER#JAMESEASTHAM")))
        .send()
        .await;

    Json(json!(res.unwrap().count()))    
}

async fn get_todo(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Json<Value> {
    let res = state.dynamodb_client
        .query()
        .table_name(&state.table_name)
        .key_condition_expression("PK = :hashKey and SK = :sortKey")
        .expression_attribute_values(":hashKey", AttributeValue::S(String::from("USER#JAMESEASTHAM")))
        .expression_attribute_values(":sortKey", AttributeValue::S(String::from(id.to_uppercase())))
        .send()
        .await;

    Json(json!(res.unwrap().count()))
}

async fn post_todo(State(state): State<Arc<AppState>>, Json(input): Json<CreateTodo>) -> Json<Value> {
    let res = state.dynamodb_client
            .put_item()
            .table_name(&state.table_name)
            .item("PK", AttributeValue::S(String::from("USER#JAMESEASTHAM")))
            .item("SK", AttributeValue::S(String::from("TESTID")))
            .item("text", AttributeValue::S(input.text))
            .send()
            .await;

    Json(json!("OK"))
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateTodo {
    text: String,
}