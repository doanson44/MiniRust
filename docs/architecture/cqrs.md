# CQRS Rules

## Commands

A command expresses business intent, not a low-level persistence operation.

Prefer:

```text
PublishPost
ChangeUserPassword
ArchiveDocument
```

Avoid generic CRUD-shaped commands such as:

```text
UpdatePost
SetStatus
UpdateRow
```

A command handler is responsible for orchestrating the write-side use case. It may load an aggregate, apply domain rules, persist the change, and emit domain events.

## Queries

A query expresses information required by a caller. Queries never change application state.

Query handlers should return purpose-built DTOs or projections. They should not expose domain aggregates as API response models.

## Domain

Aggregates enforce invariants at the write boundary. Entities and value objects model domain concepts. Domain services exist only when behavior does not naturally belong to an entity or aggregate.

The domain must remain independent from transport and infrastructure.

## Repositories

Write repositories expose aggregate-oriented persistence operations.

Read repositories expose query-oriented operations and may use SQL tailored to the requested projection.

Do not create one generic repository abstraction for every read and write operation. The read and write sides have different optimization goals.

## Transactions

A command handler defines the application transaction boundary. Persistence and domain events that belong to the same atomic operation must be committed consistently.

When asynchronous integration events are introduced, use a transactional outbox so the state change and event publication intent are persisted atomically.

## Events

### Domain events

Domain events describe facts produced inside a bounded context. They may be consumed by other parts of the same context.

### Integration events

Integration events are public contracts between bounded contexts. They must be explicitly versioned and must not expose internal persistence models.

## Consistency

The initial architecture uses one MariaDB database. CQRS still applies because the read and write models are separated logically.

If separate read storage is introduced later, document eventual-consistency behavior explicitly and make consumers idempotent.

## Idempotency and concurrency

Commands that may be retried must have an idempotency strategy appropriate to the business operation.

Aggregates that can be concurrently modified should use optimistic concurrency or another explicit concurrency policy.

## Team boundaries

Teams own bounded contexts, not shared database tables.

Cross-context access must use one of:

- an explicit application contract;
- a versioned integration event;
- a published read contract where the context intentionally provides one.

Direct access to another context's repository, aggregate, or tables is prohibited.
