---
description: Axum API handler and routing conventions for apps/api. Applied when editing apps/api files.
trigger: model_decision
globs: ["apps/api/**"]
---

# API — `minirust-api` (Axum)

## Bind address

From `minirust_config` (`ServerKind::Api`). Default: `127.0.0.1:3000`.

## Baseline routes

| Method | Path | Response |
|---|---|---|
| GET | `/health` | JSON `{"status":"ok"}` |
| GET | `/api/v1/hello` | JSON `{"message":"..."}` |

## Handler pattern

```rust
async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greetings.hello();
    (StatusCode::OK, Json(HelloResponse { message: greeting.message }))
}
```

## Rules

- Keep all routes registered in `router()`.
- Handlers extract `State`, call a service method, return HTTP. No business logic in handlers.
- JSON responses for all `/api/*` routes.
- Use `TraceLayer` for request/startup logs.
- Graceful shutdown via `tokio::signal::ctrl_c` is wired — keep it.
- `AppState` is `Clone + Send + Sync`. Services inside it must be `Clone + Send + Sync`.

## Testing

- Use `tower::ServiceExt::oneshot()` against `router(AppState::new(...))`.
- Do not spin up a live server for unit/integration tests.

```rust
#[tokio::test]
async fn health_returns_ok() {
    let app = router(AppState::new(config));
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```
