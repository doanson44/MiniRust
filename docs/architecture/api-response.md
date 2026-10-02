# API Response Contract

MiniRust uses one HTTP response contract for REST endpoints.

## Success

Successful JSON responses use the following envelope:

```json
{
  "data": {}
}
```

Rules:

- `data` contains the endpoint-specific response DTO.
- Do not put HTTP status, messages, or transport metadata inside `data`.
- HTTP status codes remain authoritative for transport semantics.
- Future pagination metadata belongs in a separate `meta` member when required; do not add it to individual DTOs.

## Errors

Errors use RFC 9457 Problem Details with media type `application/problem+json`.

MiniRust extends the standard Problem Details members with stable application error metadata:

```json
{
  "type": "https://minirust.dev/problems/user-already-exists",
  "title": "User already exists",
  "status": 409,
  "code": "USER_ALREADY_EXISTS",
  "message_key": "errors.user.already_exists",
  "locale": "vi",
  "detail": "Người dùng với email này đã tồn tại."
}
```

Rules:

- `type` identifies the problem type.
- `title` is stable for a problem type and is not used for client-side business logic.
- `status` mirrors the HTTP response status.
- `code` is the stable machine-readable application error code.
- `message_key` is the stable localization key.
- `locale` identifies the locale used for the localized `detail`.
- `detail` is user-facing text for this occurrence and must not be used as a machine-readable discriminator.
- Problem-specific extension members are allowed only when clients need structured information that cannot be represented by the standard members.
- Internal implementation details, SQL errors, connection strings, stack traces, and secrets must never be exposed.

See [api-error-code.md](api-error-code.md) for the complete error-code and i18n standard.

## Localization

The API currently supports:

- `vi` — Vietnamese
- `en` — English

The default locale is `vi`.

Clients request a locale using the standard `Accept-Language` header. Regional variants such as `vi-VN` and `en-US` are normalized to their supported base locale.

Example:

```http
Accept-Language: en-US,en;q=0.9,vi;q=0.8
```

The API selects the first supported language from the header and falls back to `vi` when no supported language is requested.

Domain and application code must return semantic errors, not localized user-facing strings. Localization belongs at the transport boundary.

The transport boundary reads its `detail` text from the shared catalog in `crates/locales`
(`locales/vi.json`, `locales/en.json`) through a typed `Key`, so no handler or response mapper
contains localized strings. `code` and `message_key` remain the machine-readable contract and are
independent of the localized wording.

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
| `413 Payload Too Large` | Request body exceeds the transport limit for the endpoint |
| `422 Unprocessable Content` | Syntactically valid request rejected by application validation |
| `429 Too Many Requests` | Rate limit exceeded |
| `500 Internal Server Error` | Unexpected server failure |
| `503 Service Unavailable` | Required infrastructure dependency is unavailable |

## CQRS boundary

API response DTOs are transport contracts. Query handlers return read DTOs/projections and command handlers return command results; neither should return Axum-specific response types.

The API layer maps those application results into this HTTP contract.

Reference: https://www.rfc-editor.org/rfc/rfc9457.html

## Integration-test contract rule

API integration tests must validate stable transport semantics rather than human-readable error wording. For error responses, tests should assert the HTTP status and, when applicable, the stable application `code` and required media type. Localized `detail` text and human-readable `title` must not be asserted in ordinary endpoint tests. Wording and localization changes must not break integration tests unless the test specifically targets the localization/response presentation contract.

Test assertion failures must not echo server error bodies or internal diagnostics. Internal database and infrastructure details belong in structured tracing, not test expectations or public API contracts.
