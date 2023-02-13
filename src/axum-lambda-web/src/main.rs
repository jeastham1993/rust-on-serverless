use std::{env, io::Write};

use aws_config::SdkConfig;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use axum::http::StatusCode;
use axum::{
    error_handling::HandleErrorLayer,
    extract::Path,
    extract::State,
    response::{IntoResponse, Json},
    routing::get,
    Form, Router,
};
use lambda_http::{aws_lambda_events::serde::Deserialize, run, Error};
use serde::Serialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[macro_use]
mod axum_ructe;

struct AppState {
    dynamodb_client: Client,
    table_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_lambda=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config: SdkConfig = aws_config::load_from_env().await;
    let dynamodb_client: Client = Client::new(&config);
    let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let shared_state = Arc::new(AppState {
        dynamodb_client: dynamodb_client,
        table_name: table_name.clone(),
    });

    let is_lambda = &env::var("LAMBDA_TASK_ROOT");

    if is_lambda.is_ok() {
        let app = Router::new()
            .route("/", get(root))
            .route("/todo", get(list_todo).post(post_todo))
            .route("/todo/:id", get(get_todo))
            // Add middleware to all layers
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(|error: BoxError| async move {
                        if error.is::<tower::timeout::error::Elapsed>() {
                            Ok(StatusCode::REQUEST_TIMEOUT)
                        } else {
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Unhandled internal error: {}", error),
                            ))
                        }
                    }))
                    .timeout(Duration::from_secs(10))
                    .layer(TraceLayer::new_for_http())
                    .into_inner(),
            )
            .with_state(shared_state);

        run(app).await;
    } else {
        let axum_app = Router::new()
            .route("/", get(root))
            .route("/todo", get(list_todo).post(post_todo))
            .route("/todo/:id", get(get_todo))
            // Add middleware to all layers
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(|error: BoxError| async move {
                        if error.is::<tower::timeout::error::Elapsed>() {
                            Ok(StatusCode::REQUEST_TIMEOUT)
                        } else {
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Unhandled internal error: {}", error),
                            ))
                        }
                    }))
                    .timeout(Duration::from_secs(10))
                    .layer(TraceLayer::new_for_http())
                    .into_inner(),
            )
            .with_state(shared_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(axum_app.into_make_service())
            .await
            .unwrap();
    }

    Ok(())
}

async fn root() -> Json<Value> {
    Json(json!({ "msg": "Welcome to the Rust ToDo API" }))
}

async fn list_todo(State(state): State<Arc<AppState>>) -> Json<Value> {
    let res = state
        .dynamodb_client
        .query()
        .table_name(&state.table_name)
        .key_condition_expression("PK = :hashKey")
        .expression_attribute_values(
            ":hashKey",
            AttributeValue::S(String::from("USER#JAMESEASTHAM")),
        )
        .send()
        .await;

    let query_result = res.unwrap();

    let mut items: Vec<Todo> = Vec::new();

    query_result
        .items()
        .expect("Items to exist")
        .into_iter()
        .for_each(|item| {
            items.push(Todo {
                id: item["id"].as_s().unwrap().to_string(),
                text: item["text"].as_s().unwrap().to_string(),
                completed: *item["completed"].as_bool().unwrap(),
            });
        });

    Json(json!(items))
}

async fn get_todo(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Json<Value> {
    let res = state
        .dynamodb_client
        .get_item()
        .table_name(&state.table_name)
        .key("PK", AttributeValue::S("USER#JAMESEASTHAM".to_string()))
        .key(
            "SK",
            AttributeValue::S(String::from(format!("TODO#{0}", id.to_uppercase()))),
        )
        .send()
        .await;

    let response_value = res.unwrap();
    let result_item = response_value.item().expect("Item should exist");

    let todo = Todo {
        id: result_item["id"].as_s().unwrap().to_string(),
        text: result_item["text"].as_s().unwrap().to_string(),
        completed: *result_item["completed"].as_bool().unwrap(),
    };

    Json(json!(todo))
}

async fn post_todo(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateTodo>,
) -> Json<Value> {
    let todo = Todo {
        completed: false,
        text: input.text,
        id: Uuid::new_v4().to_string(),
    };

    let res = state
        .dynamodb_client
        .put_item()
        .table_name(&state.table_name)
        .item("PK", AttributeValue::S(String::from("USER#JAMESEASTHAM")))
        .item(
            "SK",
            AttributeValue::S(String::from(format!("TODO#{0}", &todo.id.to_uppercase()))),
        )
        .item("text", AttributeValue::S(todo.text.to_string()))
        .item("id", AttributeValue::S(todo.id.to_string()))
        .item("completed", AttributeValue::Bool(todo.completed))
        .send()
        .await;

    Json(json!(todo))
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateTodo {
    text: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    id: String,
    text: String,
    completed: bool,
}