# API Response Contract

MiniRust uses one HTTP response contract for REST endpoints.

## Success

Successful JSON responses use the following envelope:

```json
{
  "data": {
    "message": "Hello from MiniRust"
  }
}
```

Rules:

- `data` contains the endpoint-specific response DTO.
- Do not put HTTP status, messages, or transport metadata inside `data`.
- HTTP status codes remain authoritative for transport semantics.
- Future pagination metadata belongs in a separate `meta` member when required; do not add it to individual DTOs.

## Errors

Errors use RFC 9457 Problem Details with media type `application/problem+json`. RFC 9457 standardizes machine-readable HTTP problem responses and defines `type`, `title`, `status`, and `detail` as the core members.

Reference: https://www.rfc-editor.org/rfc/rfc9457.html

Example:

```json
{
  "type": "https://minirust.dev/problems/validation-error",
  "title": "Validation error",
  "status": 422,
  "detail": "message must not be empty"
}
```

Rules:

- `type` identifies the problem type.
- `title` is stable for a problem type.
- `status` mirrors the HTTP response status.
- `detail` describes this occurrence and must help the client correct the request when applicable.
- Internal implementation details, SQL errors, connection strings, stack traces, and secrets must never be exposed.
- Add problem-specific extension members only when clients need structured information that cannot be represented by the standard members.

## Status mapping

| HTTP status | Use |
| --- | --- |
| `200 OK` | Successful read or successful command with a response body |
| `201 Created` | Successful resource creation |
| `202 Accepted` | Command accepted for asynchronous processing |
| `204 No Content` | Successful operation without a response body |
| `400 Bad Request` | Malformed request or invalid transport syntax |
| `401 Unauthorized` | Authentication required or failed |
| `403 Forbidden` | Authenticated caller is not allowed to perform the operation |
| `404 Not Found` | Requested resource does not exist |
| `409 Conflict` | Business or concurrency conflict |
| `422 Unprocessable Content` | Syntactically valid request rejected by application validation |
| `429 Too Many Requests` | Rate limit exceeded |
| `500 Internal Server Error` | Unexpected server failure |
| `503 Service Unavailable` | Required infrastructure dependency is unavailable |

## CQRS boundary

API response DTOs are transport contracts. Query handlers return read DTOs/projections and command handlers return command results; neither should return Axum-specific response types.

The API layer maps those application results into this HTTP contract.
