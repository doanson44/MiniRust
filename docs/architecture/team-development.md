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

API integration tests use an isolated MariaDB instance managed by Docker Compose. The integration test stack is defined in `docker-compose.test.yml` and waits for MariaDB's healthcheck before running Cargo tests. Integration tests are not ignored; the Docker test command is the required execution path when a MariaDB dependency is needed.

Run:

```bash
docker compose -f docker-compose.test.yml up --build --abort-on-container-exit --exit-code-from api-integration-tests
docker compose -f docker-compose.test.yml down -v
```

The test database and Cargo build/cache volumes are isolated from the normal application Compose stack.
