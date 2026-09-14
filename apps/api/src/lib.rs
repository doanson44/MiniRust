//! MiniRust REST API application.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use minirust_core::{AppError, EchoInput};
use minirust_database::Database;
use minirust_services::{EchoService, GreetingService, HealthService};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub health: HealthService,
    pub greetings: GreetingService,
    pub echo: EchoService,
    pub database: Database,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            health: HealthService,
            greetings: GreetingService,
            echo: EchoService,
            database,
        }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    echo: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

fn app_error_response(error: AppError) -> Response {
    match error {
        AppError::Validation(message) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse { error: message }),
        )
            .into_response(),
        other => {
            tracing::error!(%other, "unexpected application error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal server error".to_owned(),
                }),
            )
                .into_response()
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/hello", get(hello))
        .route("/api/v1/echo", post(echo))
        .route("/api/v1/openapi.json", get(openapi))
        .route("/swagger", get(swagger_ui))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match state.database.health().await {
        Ok(()) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "ok",
            }),
        ),
        Err(error) => {
            tracing::error!(%error, "database health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "degraded",
                    database: "unavailable",
                }),
            )
        }
    }
}

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greetings.hello();
    (
        StatusCode::OK,
        Json(HelloResponse {
            message: greeting.message,
        }),
    )
}

async fn echo(State(state): State<AppState>, Json(body): Json<EchoRequest>) -> impl IntoResponse {
    match EchoInput::parse(body.message) {
        Ok(input) => (
            StatusCode::OK,
            Json(EchoResponse {
                echo: state.echo.echo(input).echo,
            }),
        )
            .into_response(),
        Err(error) => app_error_response(error),
    }
}

async fn openapi() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "MiniRust API", "version": "0.1.0" },
            "paths": {
                "/health": { "get": { "summary": "Health and MariaDB connectivity", "responses": { "200": { "description": "Application and database are healthy" }, "503": { "description": "Database is unavailable" } } } },
                "/api/v1/hello": { "get": { "summary": "Hello", "responses": { "200": { "description": "Greeting" } } } },
                "/api/v1/echo": { "post": { "summary": "Echo a message", "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["message"], "properties": { "message": { "type": "string" } } } } } }, "responses": { "200": { "description": "Echo response" }, "422": { "description": "Validation error" } } } }
            }
        })),
    )
}

async fn swagger_ui() -> impl IntoResponse {
    let html = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>MiniRust API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>window.onload=()=>SwaggerUIBundle({url:'/api/v1/openapi.json',dom_id:'#swagger-ui'});</script></body></html>"#;
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
}
