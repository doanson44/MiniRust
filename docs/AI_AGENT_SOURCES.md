# MiniRust AI Agent Sources

This document defines the source hierarchy AI Agent must use when working on MiniRust.

## 1. Repository Source of Truth

Repository:
- GitHub: https://github.com/doanson44/MiniRust
- Default branch: master

The live GitHub repository is the authoritative source for the current implementation state.
Before making claims about implementation, debugging, architecture, testing, or deployment, inspect GitHub first.

Important entry points:
- README.md
- Cargo.toml
- apps/
- crates/
- docs/
- Docker and Compose files
- CI workflows
- integration tests
- database migrations

## 2. Project Architecture Sources

Use these repository documents for architectural intent:
- docs/architecture/README.md
- docs/architecture/cqrs.md
- docs/architecture/team-development.md
- docs/architecture/api-response.md

These documents describe intended architecture; they do not prove that every described feature is implemented.
If documentation and implementation disagree, report the discrepancy explicitly.

## 3. Current Repository Baseline

The current repository baseline indicates:
- Rust workspace
- Axum API
- Leptos SSR web application
- MariaDB
- SQLx with the MySQL/MariaDB driver
- CQRS-oriented application boundary
- commands/ and queries/ separation
- Tailwind CSS
- Docker Compose
- database health checks
- REST API response envelope
- RFC 9457 Problem Details for API errors

Always verify the exact current state against GitHub.

## 4. External Technical Sources

Prefer official or primary sources when external technical documentation is required.

### Rust
- The Rust Programming Language: https://doc.rust-lang.org/book/
- Rust Reference: https://doc.rust-lang.org/reference/
- Rust Standard Library: https://doc.rust-lang.org/std/
- Cargo Book: https://doc.rust-lang.org/cargo/
- Rustdoc Book: https://doc.rust-lang.org/rustdoc/

### Tokio
- Tokio documentation: https://docs.rs/tokio/

### Axum
- Axum documentation: https://docs.rs/axum/
- Axum repository: https://github.com/tokio-rs/axum

### Leptos
- Leptos Book: https://book.leptos.dev/
- Leptos API documentation: https://docs.rs/leptos/

### SQLx
- SQLx documentation: https://docs.rs/sqlx/
- SQLx repository: https://github.com/launchbadge/sqlx/
- Use the MySQL/MariaDB driver for MiniRust.

### MariaDB
- MariaDB documentation: https://mariadb.com/docs/

### Docker
- Docker documentation: https://docs.docker.com/
- Docker Compose documentation: https://docs.docker.com/compose/

### Tailwind CSS
- Tailwind CSS documentation: https://tailwindcss.com/docs
- Tailwind is the project's CSS framework. Do not introduce Bootstrap.

### HTTP API Errors
- RFC 9457: https://www.rfc-editor.org/rfc/rfc9457.html

### GitHub
- GitHub documentation: https://docs.github.com/

## 5. Source Precedence

For current implementation facts:
1. Current GitHub repository state
2. Verified CI/test/build results
3. Repository architecture documentation
4. User's current task intent
5. External official documentation
6. General model knowledge

For architectural intent:
1. User's current explicit architectural decision
2. Current repository architecture documentation
3. Existing implementation
4. External technical guidance
5. General model knowledge

Never silently reconcile conflicting sources.

## 6. Source Discipline

If a source does not support a claim:
- say so;
- inspect GitHub if the claim concerns implementation;
- consult official documentation if the claim concerns tool/library behavior;
- never invent missing details.

When version compatibility matters, verify the exact dependency version in Cargo.toml or Cargo.lock.

## 7. Legacy Sources

Older material may mention PostgreSQL, Redis, SQL Server, Tiberius, or Bootstrap.
Do not treat those as current MiniRust technology choices unless the current repository explicitly uses them.

## 8. Deployment Direction

The target deployment model is one hosting/deployment unit for the web and API.
This is an architectural direction, not proof that the current repository already implements it.
For deployment tasks, inspect the current runtime topology, routing, Docker configuration, and process model before making changes.