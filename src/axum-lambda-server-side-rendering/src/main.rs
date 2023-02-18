pub mod auth;
mod services;

use std::{env, io::Write};

use auth::auth::AuthService;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{
    error_handling::HandleErrorLayer, extract::State, response::IntoResponse, routing::get, Form,
    Router,
};
use lambda_http::{run, Error};
use services::services::{CompleteTodo, CreateTodo, DeleteTodo, LoginCommand, TodoService};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::services::services::TodoHomePageView;

#[macro_use]
mod axum_ructe;

macro_rules! configure_routes {
    () => {
        // The macro will expand into the contents of this block.
        Router::new()
            .route("/login", get(login).post(login_post))
            .route("/home", get(home_page).post(home_page_post))
            .route("/complete", get(home_page).post(complete_todo))
            .route("/delete-todo", get(home_page).post(delete_todo))
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
    };
}

struct AppState {
    todo_service: TodoService,
    auth_service: AuthService,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_lambda=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config: SdkConfig = aws_config::load_from_env().await;
    let auth_client: Client = Client::new(&config);
    let todo_client: Client = Client::new(&config);
    let table_name = &env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let shared_state = Arc::new(AppState {
        todo_service: TodoService::new(todo_client, table_name.to_string()),
        auth_service: AuthService::new(auth_client, table_name.to_string()),
    });

    let is_lambda = &env::var("LAMBDA_TASK_ROOT");

    if is_lambda.is_ok() {
        let routes: Router<Arc<AppState>, lambda_http::Body> = configure_routes!();
        let app = routes.with_state(shared_state);

        run(app).await;
    } else {
        let routes: Router<Arc<AppState>, _> = configure_routes!();
        let axum_app = routes.with_state(shared_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(axum_app.into_make_service())
            .await
            .unwrap();
    }

    Ok(())
}

/// Home page handler; just render a template with some arguments.
async fn home_page(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let user = cookies
        .get("username")
        .and_then(|c| c.value().parse().ok())
        .unwrap();

    let items = state.todo_service.list_todos(user).await;

    render!(
        templates::page_html,
        TodoHomePageView {
            active: items.clone()
                .into_iter()
                .filter(|todo| !todo.completed)
                .collect(),
            completed: items
                .into_iter()
                .filter(|todo| todo.completed)
                .collect()
        }
    )
}

async fn home_page_post(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    form: Form<CreateTodo>,
) -> impl IntoResponse {
    tracing::debug!("Creating {}", form.text.clone());

    let user = cookies
        .get("username")
        .and_then(|c| c.value().parse().ok())
        .unwrap();

    state.todo_service.create_todo(user, form.0).await;

    Redirect::to("/home")
}

/// Login handler
async fn login() -> impl IntoResponse {
    render!(templates::login_html, String::from(""))
}

async fn login_post(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    form: Form<LoginCommand>,
) -> impl IntoResponse {
    tracing::debug!("Logging in {}", form.username);

    let environment_password = &env::var("PASSWORD").unwrap().to_string();

    if environment_password == &form.password {
        tracing::debug!("Auth is valid");

        let session_token = state.auth_service.generate_session().await;

        cookies.add(Cookie::new("authentication", form.username.clone()));
        cookies.add(Cookie::new("username", form.username.clone()));
        cookies.add(Cookie::new("session_token", session_token));

        tracing::debug!("Login success");

        Redirect::to("/home")
    } else {
        tracing::debug!("Login failed, redirecting to login");
        Redirect::to("/login")
    }
}

async fn complete_todo(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    form: Form<CompleteTodo>,
) -> impl IntoResponse {
    let user = cookies
        .get("username")
        .and_then(|c| c.value().parse().ok())
        .unwrap();

    state.todo_service.complete(user, &form.id).await;

    Redirect::to("/home")
}

async fn delete_todo(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    form: Form<DeleteTodo>,
) -> impl IntoResponse {
    let user = cookies
        .get("username")
        .and_then(|c| c.value().parse().ok())
        .unwrap();

    state.todo_service.delete_todo(user, &form.id).await;

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
