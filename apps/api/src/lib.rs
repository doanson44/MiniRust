//! MiniRust REST API application.

mod response;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use minirust_core::AppError;
use minirust_database::Database;
use minirust_services::cqrs::{CommandHandler, QueryHandler};
use minirust_services::{EchoCommand, EchoCommandHandler, GreetingQuery, GreetingQueryHandler};
use response::{ApiResponse, ProblemDetails};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub echo: EchoCommandHandler,
    pub greeting: GreetingQueryHandler,
    pub database: Database,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            echo: EchoCommandHandler,
            greeting: GreetingQueryHandler,
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

fn app_error_response(error: AppError) -> ProblemDetails {
    match error {
        AppError::Validation(message) => ProblemDetails::validation(message),
        other => {
            tracing::error!(%other, "unexpected application error");
            ProblemDetails::internal()
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
            Json(ApiResponse::new(HealthResponse {
                status: "ok",
                database: "ok",
            })),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "database health check failed");
            ProblemDetails::service_unavailable("Database is unavailable").into_response()
        }
    }
}

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greeting.handle(GreetingQuery);
    (
        StatusCode::OK,
        Json(ApiResponse::new(HelloResponse {
            message: greeting.message,
        })),
    )
}

async fn echo(State(state): State<AppState>, Json(body): Json<EchoRequest>) -> impl IntoResponse {
    match state.echo.handle(EchoCommand {
        message: body.message,
    }) {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::new(EchoResponse { echo: result.echo })),
        )
            .into_response(),
        Err(error) => app_error_response(error).into_response(),
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
                "/api/v1/hello": { "get": { "summary": "Hello query", "responses": { "200": { "description": "Greeting" } } } },
                "/api/v1/echo": { "post": { "summary": "Echo command", "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["message"], "properties": { "message": { "type": "string" } } } } } }, "responses": { "200": { "description": "Echo response" }, "422": { "description": "Validation error" } } } }
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
