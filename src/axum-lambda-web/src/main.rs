mod implementations;
mod application;

use std::{env};

use aws_config::{BehaviorVersion, SdkConfig, Region};
use axum::{extract::Path, extract::State, response::{Json}, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{
    run,
    Error,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use crate::application::application::{AppState, create_todo, CreateTodo, Todo, ToDoRepo};
use crate::implementations::implementations::{DynamoDbToDoRepo, InMemoryToDoRepo};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_lambda=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let is_lambda = &env::var("LAMBDA_TASK_ROOT");

    let config: SdkConfig = aws_config::load_defaults(BehaviorVersion::latest()).await;

    if is_lambda.is_ok() {
        let dynamodb_client: Client = Client::new(&config);
        let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");

        let shared_state = Arc::new(AppState {
            todo_repo: Arc::new(DynamoDbToDoRepo::new(dynamodb_client.clone(), table_name.clone()))
        });

        let app = Router::new()
            .route("/", get(root))
            .route("/todo", get(list_todo).post(create_todo))
            .route("/todo/:id", get(get_todo))
            .with_state(shared_state);
        run(app).await;
    } else {
        let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
            .endpoint_url(
                // 8000 is the default dynamodb port
                "http://localhost:8000",
            )
            .region(Region::from_static("us-east-1"))
            .build();
        let local_client = Client::from_conf(dynamodb_local_config);
        let local_table_name = String::from("TODO");

        let local_state = Arc::new(AppState {
            todo_repo: Arc::new(DynamoDbToDoRepo::new(local_client.clone(), local_table_name))
        });

        let axum_app = Router::new()
            .route("/", get(root))
            .route("/todo", get(list_todo).post(post_todo))
            .route("/todo/:id", get(get_todo))
            .with_state(local_state);

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
    let items = state.todo_repo.list("JAMESEASTHAM").await.unwrap();

    Json(json!(items))
}

async fn get_todo(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Json<Value> {
    let todo = state.todo_repo.get(&id).await.unwrap();

    Json(json!(todo.clone()))
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

    let _ = state.todo_repo.create(todo).await;

    Json(json!("OK"))
}