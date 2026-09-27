# MiniRust Entity and Identity Design

## Identity strategy

MiniRust uses UUIDv7 for domain entity identity.

The current Rust implementation uses the `uuid` crate with only the `v7` generation feature enabled. UUIDv7 is generated at the application/domain boundary rather than by MariaDB.

## Entity contract

The base entity abstraction is intentionally minimal:

- an entity has an identity;
- the identity is strongly typed;
- entity identity is independent of persistence;
- audit fields are not part of the base entity.

The `Entity` trait exposes the entity's concrete ID type. Concrete bounded contexts should define newtypes such as `UserId` and `OrderId` instead of using a shared UUID type directly.

## Aggregate roots

`AggregateRoot` is a separate marker contract over `Entity`.

An aggregate root is the externally addressable entry point for an aggregate. Child entities remain owned by the aggregate and should not receive generic repositories merely because they implement `Entity`.

## ID representation

The shared `EntityId` is a UUIDv7-backed primitive. Bounded-context IDs should wrap it:

```rust
pub struct UserId(EntityId);
pub struct OrderId(EntityId);
```

This preserves compile-time separation between semantically different identifiers even though they use the same UUID representation.

## Persistence boundary

Domain entities and IDs must not depend on SQLx or MariaDB.

Infrastructure is responsible for mapping IDs to the database representation. Database models remain separate from domain entities.

## CQRS interaction

Commands and aggregate operations use domain entities and strongly typed IDs.

Queries should normally return purpose-built projections/DTOs and should not expose domain entities merely to satisfy transport responses.

## Lifecycle and audit

The base entity does not contain:

- `created_at`
- `updated_at`
- `deleted_at`
- `version`
- tenant or persistence metadata

These concerns are added only where a specific domain or concurrency requirement justifies them.

## Evolution

UUIDv7 keeps identity generation independent of the current single-MariaDB deployment model. If persistence or infrastructure changes later, domain identity does not need to become database-generated.
