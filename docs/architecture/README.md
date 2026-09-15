# MiniRust Architecture

MiniRust is designed as a **CQRS-oriented modular monolith** with domain boundaries that can evolve into independently deployable services when operational or scaling requirements justify the split.

## Core principles

1. **CQRS is an application boundary.** Commands change state; queries read state.
2. **Domain logic is isolated.** Domain code must not depend on Axum, Leptos, SQLx, MariaDB, or vendor SDKs.
3. **Bounded contexts own their models.** A context must not reach into another context's tables, repositories, aggregates, or internal DTOs.
4. **Handlers are transport adapters.** HTTP and SSR handlers translate requests into commands or queries and map results into transport responses.
5. **Write and read models are independent.** They may share MariaDB initially; they must not share application responsibilities.
6. **Events are contracts, not implementation details.** Domain events remain internal to a context; integration events are versioned contracts between contexts.
7. **Distributed infrastructure is evolutionary.** Messaging, separate read stores, outbox processing, and service extraction are enabled by the boundaries but are not required for the first deployment.

## Current implementation

The repository currently uses the existing `crates/services` package as the application-layer compatibility boundary. Its internal structure is CQRS-oriented under `commands/` and `queries/`. The package name is intentionally unchanged in this migration so the existing lockfile and workspace remain stable.

The next structural migration may rename that package to `minirust-application` once lockfile regeneration and local CI verification are available.

## Target shape

```text
MiniRust/
├── apps/
│   ├── api/                 # Axum transport
│   └── web/                 # Leptos SSR transport
│
├── crates/
│   ├── config/              # configuration boundary
│   ├── core/                # domain primitives and application errors
│   ├── services/            # current application-layer package; CQRS modules
│   │   ├── commands/
│   │   └── queries/
│   └── database/            # MariaDB/SQLx infrastructure
│
├── docs/
│   └── architecture/
│
└── migrations/              # database schema migrations as persistence grows
```

## Request flow

### Command

```text
HTTP request
  ↓
Axum handler
  ↓
Command
  ↓
Command handler
  ↓
Aggregate / domain rules
  ↓
Write repository
  ↓
MariaDB transaction
```

### Query

```text
HTTP/SSR request
  ↓
Transport handler
  ↓
Query
  ↓
Query handler
  ↓
Read repository / projection
  ↓
Read DTO
```

The query side must not mutate state and should not require the write-side domain model merely to format a response.

## Scaling path

```text
Level 1: CQRS in one process + one MariaDB
    ↓
Level 2: optimized read projections in the same database
    ↓
Level 3: transactional outbox + asynchronous integration events
    ↓
Level 4: separate read store where justified
    ↓
Level 5: extract bounded contexts into independently deployed services
```

Do not jump directly to Level 5. Boundaries must be proven through real feature development first.
