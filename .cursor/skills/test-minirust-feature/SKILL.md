---
name: test-minirust-feature
description: Validates MiniRust changes with focused tests and mandatory HTTP endpoint integration coverage.
---

# Test a MiniRust feature

## Endpoint coverage gate

For every HTTP endpoint introduced or modified:

1. Locate the route registration.
2. Locate its integration test.
3. Confirm the test exercises the real Axum router with `router().oneshot(...)`.
4. Confirm success and meaningful error/validation paths are covered.
5. Confirm MariaDB-backed endpoints use the Docker-backed integration environment when infrastructure is required.
6. Report any uncovered endpoint as a validation failure.

Unit tests do not satisfy this endpoint coverage gate.

## Validation

Run the repository's configured formatting, compilation, linting, and test checks. Do not claim a check passed unless it was actually executed.
