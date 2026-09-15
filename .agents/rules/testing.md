---
description: Endpoint integration-testing requirements for the MiniRust API.
trigger: model_decision
globs: ["apps/api/**", "crates/**"]
---

# Endpoint Integration Testing

Every HTTP endpoint introduced or modified MUST have an integration test.

## Required boundary

The test MUST exercise the endpoint through the real Axum router:

```text
HTTP request
  ↓
Axum router
  ↓
Transport handler
  ↓
Command / Query handler
  ↓
Repository / adapter
  ↓
MariaDB / infrastructure when applicable
  ↓
HTTP response
```

Calling a handler function directly is not sufficient for endpoint coverage.

## Required coverage

For each endpoint, cover as applicable:

- HTTP method and route registration.
- Success status code.
- Response body and response contract.
- Relevant response headers.
- Validation and meaningful client-error paths.
- Meaningful server-error paths.
- Real MariaDB behavior when the endpoint depends on database infrastructure.

## CQRS rule

Unit tests for commands, queries, handlers, domain logic, or repositories do NOT replace endpoint integration tests.

## Definition of done

A feature that adds or changes an HTTP endpoint is incomplete until its integration test is added or updated and included in the normal workspace test suite.

During implementation and review, compare registered HTTP routes with integration tests and identify any uncovered endpoint.

If an endpoint is exempted for a concrete technical reason, document the exception and rationale in the change.

## Infrastructure

Prefer real infrastructure for integration tests when practical. MariaDB-backed tests should use the Docker-backed integration environment defined for MiniRust. Keep tests isolated, deterministic, and responsible for cleaning up test infrastructure.

Never claim a test or validation check passed unless it was actually executed.
