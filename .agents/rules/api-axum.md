---
description: Axum API transport conventions for apps/api.
trigger: model_decision
globs: ["apps/api/**"]
---

# API — `minirust-api` (Axum)

## Bind address

From `minirust_config` (`ServerKind::Api`). Default: `127.0.0.1:3000`.

## Handler responsibility

Handlers are transport adapters only.

```text
HTTP request
  ↓
Axum handler
  ↓
Command or Query
  ↓
Application handler
  ↓
HTTP response contract
```

Do not put business rules, repository calls, or SQL in the Axum handler.

## Routes

| Method | Path | Role |
|---|---|---|
| GET | `/health` | Infrastructure health check |
| GET | `/api/v1/hello` | Query |
| POST | `/api/v1/echo` | Command |

## Response contract

Successful JSON API responses use `{ "data": ... }`.

Errors use RFC 9457 Problem Details with `Content-Type: application/problem+json`.

```json
{
  "type": "https://minirust.dev/problems/validation-error",
  "title": "Validation error",
  "status": 422,
  "detail": "message must not be empty"
}
```

HTTP status codes remain authoritative. Do not create endpoint-specific error envelopes.

See `docs/architecture/api-response.md` for the complete contract and status mapping.

## Rules

- Register all routes in `router()`.
- Map transport input into a typed command/query.
- Map application errors to the standardized HTTP response contract at the transport boundary.
- JSON responses for `/api/*` routes.
- Use `TraceLayer` for request tracing.
- Keep graceful shutdown in the binary.

## Testing

Use `tower::ServiceExt::oneshot()` against the router. Do not start a live server for handler tests.
