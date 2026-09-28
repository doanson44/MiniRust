# Team Development Rules

MiniRust is expected to support a large engineering organization and a very large feature set. Architecture therefore optimizes for ownership boundaries and low coordination cost.

## Ownership

- A team owns one or more bounded contexts.
- A bounded context owns its domain model, application handlers, persistence model, and integration contracts.
- Shared crates contain only genuinely cross-cutting primitives. They must not become a second business-domain monolith.

## Feature placement

For every feature, identify its bounded context first.

Then place the implementation inside that context:

```text
context/
├── domain/
├── application/
│   ├── commands/
│   └── queries/
├── infrastructure/
│   └── persistence/
└── contracts/
```

The current baseline uses `crates/services` as the application-layer compatibility package. New feature work should follow the same internal command/query separation rather than adding another generic service collection.

## Dependency rules

Allowed direction:

```text
Transport → Application → Domain
Infrastructure → Application contracts / Domain
```

Forbidden direction:

```text
Domain → Axum
Domain → Leptos
Domain → SQLx
Domain → MariaDB
Domain → transport DTOs
```

A query must not call a command handler. A command handler must not call a query handler merely to obtain display data.

## Cross-context rules

Do not import another context's private modules.

Do not query another context's tables directly.

Do not share domain entities between contexts just because their fields look similar.

Prefer explicit contracts and translation at the boundary.

## Review checklist

Every feature review should answer:

1. Which bounded context owns the feature?
2. Is the operation a command or a query?
3. Does the handler contain transport logic only?
4. Are domain invariants enforced on the write side?
5. Is the read model purpose-built for the query?
6. Are cross-context dependencies explicit?
7. Is transaction scope clear?
8. Is retry/idempotency behavior clear for retriable commands?
9. Are events versioned when they cross a context boundary?
10. Can the context eventually be extracted without rewriting its domain model?


## Integration test execution

Every new API endpoint must have integration-test coverage in the same change. The test must exercise the public HTTP contract through the Axum router and the real application/database boundary; do not satisfy this rule with only unit tests.

Coverage is intentionally business-rule focused. Add the smallest set of integration tests that proves the endpoint's meaningful business invariants, authorization rules, persistence behavior, and important response contract. Do not create one test for every trivial status-code permutation, serialization detail, or mechanically duplicated endpoint path when the behavior is already covered by a stronger scenario.

For each API change, review existing integration tests first and extend them when an existing scenario can cover the new rule. Prefer a small number of scenario-oriented tests over endpoint-count-driven test proliferation.

API integration tests that require MariaDB use Testcontainers. Each integration-test fixture starts an isolated MariaDB container, waits until it is ready, runs migrations and seeds test data, then the container is removed when the fixture is dropped.

Integration tests are not ignored and do not require a separate test Compose command. Docker is the infrastructure prerequisite; the test process owns the container lifecycle.

Run the normal workspace test command:

```bash
cargo test --workspace --all-targets
```

Test workflow for API work:

1. Implement the endpoint and its application/domain rules.
2. Identify the business rules that must hold at the HTTP boundary.
3. Add or extend the smallest integration-test scenarios that prove those rules.
4. Run the relevant integration test target.
5. Run workspace verification when practical.
6. Review the diff and confirm the test covers behavior, not implementation details.

A missing integration test is incomplete API work unless the endpoint is explicitly documented as an exception by architecture or task scope.


## Transaction boundary implementation

The application command handler defines the logical transaction boundary: one command is the unit of state change and must not be split across unrelated repository calls without an explicit consistency decision.

The infrastructure repository owns the physical SQL transaction lifecycle for that command operation. This keeps SQLx and MariaDB details out of the application layer while preserving atomicity at the command boundary. If a future command composes multiple persistence operations, its repository contract must expose one atomic command operation rather than having the transport or query layer coordinate transactions.


## Web rendering and hydration

The web application uses Leptos full-stack SSR with client-side hydration. The server renders the initial HTML through Axum and `leptos_axum`; the browser loads the WASM bundle produced by `cargo-leptos` and hydrates the same component tree. Interactive UI behavior belongs in Leptos components rather than inline JavaScript. The web and API remain separate processes behind the same Caddy ingress.


## CI verification and local-environment fallback

GitHub Actions is the authoritative remote verification path when local execution cannot provide a trustworthy result. The repository CI workflow must verify, at minimum:

1. `cargo fmt --all -- --check`.
2. `cargo check --workspace --all-targets`.
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
4. `cargo test --workspace --all-targets`.
5. `cargo leptos build --release` with the `wasm32-unknown-unknown` target for the SSR + hydration web application.

API integration tests may start their own MariaDB Testcontainers; CI runners must provide Docker for those tests.

When local DNS/network restrictions prevent dependency resolution or repository checkout, do not report local build/test verification as passed. Inspect the corresponding GitHub Actions workflow run for the commit instead. A GitHub Actions result is authoritative evidence only for the exact commit it ran against.

If no CI workflow exists, repository verification is incomplete for changes that require compilation, tests, linting, or frontend build validation; add the CI workflow before treating remote verification as available.
\n## API integration-test assertion policy\n\nAPI integration tests verify the public contract, not presentation wording or internal diagnostics.\n\n- Always assert the HTTP status code for the scenario.\n- Assert the stable application error `code` when the API contract defines one and the scenario depends on that error.\n- Assert `Content-Type: application/problem+json` when verifying an error response contract.\n- Do not assert localized `detail`, human-readable `title`, or other presentation text unless the test is specifically for the response-contract/localization behavior.\n- Do not include server-generated error messages, SQL errors, stack traces, or internal diagnostics in assertion messages.\n- Prefer business-rule and contract assertions over implementation details.\n\nThis keeps integration tests stable when wording or localization changes while preserving the machine-readable API contract.\n