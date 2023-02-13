mod services;

use std::{env, io::Write};

use aws_config::SdkConfig;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    response::{IntoResponse, Json},
    routing::get,
    Form, Router,
};
use lambda_http::{aws_lambda_events::serde::Deserialize, run, Error};
use serde_json::{json, Value};
use services::services::{Todo, CreateTodo, TodoService};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[macro_use]
mod axum_ructe;

struct AppState {
    todo_service: TodoService
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
        todo_service: TodoService::new(dynamodb_client, table_name.to_string())
    });

    let is_lambda = &env::var("LAMBDA_TASK_ROOT");

    if is_lambda.is_ok() {
        let app = Router::new()
            .route("/", get(root))
            .route("/home", get(home_page).post(home_page_post))
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
            .layer(CookieManagerLayer::new())
            .with_state(shared_state);

        run(app).await;
    } else {
        let axum_app = Router::new()
            .route("/", get(root))
            .route("/home", get(home_page).post(home_page_post))
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
            .layer(CookieManagerLayer::new())
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

/// Home page handler; just render a template with some arguments.
async fn home_page(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let items = state.todo_service.list_todos().await;

    render!(templates::page_html, items)
}

async fn home_page_post(
    State(state): State<Arc<AppState>>,
    form: Form<CreateTodo>,
) -> impl IntoResponse {
    tracing::debug!("Creating {}", form.text.clone());

    state.todo_service.create_todo(form.0).await;

    Redirect::to("/home")
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn nav(out: &mut impl Write) -> std::io::Result<()> {
    templates::nav_html(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("axum", "https://crates.io/crates/axum"),
        ],
    )
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
