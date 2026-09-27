# API Error Code Standard

## Purpose

This document defines the system-wide application error code and localization rules for MiniRust REST APIs.

The HTTP status remains the transport-level semantic. The application `code` identifies the specific failure. Clients must not infer application behavior from localized `detail` text.

## Problem Details

Every API error uses:

```text
application/problem+json
```

The standard contract is:

```json
{
  "type": "https://minirust.dev/problems/...",
  "title": "Stable problem title",
  "status": 422,
  "code": "VALIDATION_FAILED",
  "message_key": "errors.validation.failed",
  "locale": "vi",
  "detail": "Dữ liệu không hợp lệ."
}
```

### Field rules

| Field | Rule |
| --- | --- |
| `type` | Stable URI identifying the problem type |
| `title` | Stable problem title |
| `status` | HTTP status code |
| `code` | Stable machine-readable application error code |
| `message_key` | Stable localization key |
| `locale` | Locale used for `detail` |
| `detail` | Localized user-facing message |

## Error code ownership

System/common errors are defined by the shared API/application contract.

Business errors are owned by the bounded context that produces them.

Examples:

```text
VALIDATION_FAILED
BAD_REQUEST
AUTHENTICATION_REQUIRED
AUTHENTICATION_FAILED
FORBIDDEN
RESOURCE_NOT_FOUND
RATE_LIMITED
INTERNAL_ERROR
DEPENDENCY_UNAVAILABLE

USER_ALREADY_EXISTS
USER_NOT_FOUND
ORDER_NOT_FOUND
ORDER_ALREADY_CANCELLED
```

Do not encode the HTTP status into `code`. For example, use `USER_ALREADY_EXISTS`, not `409`.

## Localization

Supported locales:

```text
vi
en
```

Default:

```text
vi
```

The API reads `Accept-Language` and normalizes regional variants to the supported base locale.

The translation key is stable across locales:

```text
errors.user.already_exists
```

Example translations:

```text
vi: Người dùng với email này đã tồn tại.
en: A user with this email already exists.
```

Do not branch client behavior on `detail`. Use `code` or `message_key`.

## Domain/application boundary

Domain and application layers must not depend on HTTP, Axum, or a concrete user locale.

Prefer semantic errors:

```text
UserAlreadyExists
MessageRequired
MessageTooLong
```

The transport layer maps them to:

```text
semantic error
  -> application error code
  -> message key
  -> requested locale
  -> localized detail
```

This keeps localization outside domain logic and preserves the dependency direction:

```text
Transport -> Application -> Domain
Infrastructure -> Application contracts / Domain
```

## Validation

Validation failures use HTTP `422` and `VALIDATION_FAILED` as the general category when appropriate. Field-specific validation errors expose stable field-level codes and message keys.

Example:

```json
{
  "type": "https://minirust.dev/problems/validation-error",
  "title": "Validation error",
  "status": 422,
  "code": "MESSAGE_REQUIRED",
  "message_key": "errors.validation.message_required",
  "locale": "vi",
  "detail": "Message không được để trống."
}
```

A validation error may add structured extension members when a client needs multiple field failures.

## Security

Never expose SQL errors, stack traces, credentials, connection strings, internal paths, or vendor-specific infrastructure details in Problem Details.

Internal diagnostics belong in structured tracing/logging.

## Compatibility

Adding a new error code is backward-compatible when existing codes retain their meaning.

Changing the meaning of an existing code is a breaking API change.

Changing localized wording is not a change to the machine-readable contract.

Adding a new locale does not change existing error codes or message keys.


## User administration codes

The admin user management API uses these stable codes:

```text
INVALID_EMAIL
INVALID_ROLE
USER_NOT_FOUND
EMAIL_ALREADY_EXISTS
PROTECTED_USER
FORBIDDEN
```

`FORBIDDEN` is returned when the caller is authenticated but does not have the admin system role.
