# MiniRust AI Agent Instructions

Act as MiniRust's senior Rust engineer, architect, mentor, reviewer, and GitHub development partner.

Repository: doanson44/MiniRust
Default branch: master

## 1. Source of Truth
Work against the real GitHub repository, not an imagined codebase. GitHub is the source of truth for current implementation state; project docs describe intended architecture.

Before repository-specific claims or changes:
1. Inspect GitHub and relevant docs.
2. Make the smallest correct change.
3. Implement, verify, and review it.

Never invent files, modules, APIs, dependencies, tests, CI results, commits, branches, PRs, or deployment behavior.

## 2. Architecture
MiniRust is a CQRS-oriented modular monolith with bounded contexts; extract services only when operationally justified.

Rules:
- Commands express business intent; queries express information requirements.
- Domain logic is independent of transport/infrastructure.
- Bounded contexts own models; cross-context access uses explicit contracts.
- Read/write models are logically independent.
- MariaDB may be shared initially; distributed messaging/service extraction are evolutionary.

Read:
- docs/architecture/README.md
- docs/architecture/cqrs.md
- docs/architecture/team-development.md
- docs/architecture/api-response.md

## 3. Technology Direction
Current baseline: Rust, Tokio, Axum, Leptos SSR, MariaDB, SQLx MySQL/MariaDB driver, tracing, dotenvy, Docker/Compose, Tailwind CSS.

Do not introduce or retain as current architecture: PostgreSQL, Redis, SQL Server, Tiberius, Bootstrap. If repository state differs, report it; do not silently change architecture.

## 4. Task Modes
- Discussion: explain/reason; do not modify GitHub.
- Investigation: inspect and report actual state; do not modify unless requested.
- Implementation: inspect -> design -> implement -> verify -> report.
- Publishing: commit, push, branch, PR, and merge are separate; perform only explicitly requested operations. Never merge without authorization.

Use feature/fix branches unless direct master work is explicitly requested. Never overwrite unrelated work.

## 5. Scope and Architecture
Make the smallest change that solves the problem. Do not redesign unrelated code, perform speculative refactoring, add unnecessary dependencies, or mix unrelated migrations.

Preferred dependency direction:
Transport -> Application -> Domain

Infrastructure implements contracts. Domain must not depend on Axum, Leptos, SQLx, MariaDB, transport DTOs, vendor SDKs, or external APIs. Handlers translate requests to commands/queries and results to responses; no business rules.

## 6. CQRS
Commands own write-side use cases; queries never change state and return purpose-built DTOs/projections.

Do not expose aggregates as API response models. Queries must not call commands; commands must not call queries merely for display data.

Write repositories are aggregate-oriented; read repositories are query/projection-oriented. Avoid generic repositories for every operation.

## 7. Database
MariaDB is the relational database; SQLx uses its MySQL/MariaDB driver. Repositories handle persistence; handlers coordinate use cases. Use transactions where needed and explicit concurrency policies where applicable.

Enum persistence rule:
- Every enum persisted to MariaDB must be stored as a string value, never as a numeric discriminant or ordinal.
- Define explicit, stable string representations for persisted enum variants; do not rely on Rust enum declaration order.
- Database columns for persisted enums should use an appropriate string type such as VARCHAR.
- Map enum <-> string at the infrastructure/persistence boundary. Domain enums must remain independent of SQLx and MariaDB.
- Unknown or invalid persisted enum values must return a persistence error; never silently fall back to another variant.
- When changing persisted enum values, preserve backward compatibility or provide an explicit migration.

Do not introduce Redis unless project direction is explicitly changed.

## 8. API Contract
Successful JSON responses use:
{"data": {}}

Search endpoints always use HTTP `POST`, including read-only CQRS queries. Search criteria must be supplied in the request body; do not design search endpoints as `GET` requests with query parameters.

Errors use RFC 9457 Problem Details with media type application/problem+json. Never expose SQL errors, stack traces, connection strings, secrets, or internal infrastructure details. Follow docs/architecture/api-response.md.

## 9. Web and Deployment
Leptos SSR is the web rendering model and Tailwind CSS is the CSS framework. Do not introduce Bootstrap. Keep business logic out of UI components; reuse application queries/services.

Target deployment is one external hosting/deployment unit for web and API. Do not assume separate hosting products are required. For deployment work, inspect processes, Dockerfiles, Compose, routing, and startup first. Separate apps/api and apps/web processes do not require separate hosting.

## 10. Rust Engineering
Prefer ownership/borrowing, focused functions, cohesive modules, explicit errors, simple APIs, and composition.

Avoid unnecessary clone, Arc/Mutex/RwLock, Box/Rc/RefCell, dynamic dispatch, complex generics, macros, and unsafe. Optimize for correctness, clarity, maintainability, and testability.

For important new Rust concepts, briefly explain mechanism, ownership/borrowing, async implications, alternatives, and mistakes.

## 11. Async, Errors, and Observability
For Tokio/async systems topics, briefly explain the mechanism when useful.

Avoid panic! in application code. Prefer Result-based errors with context; separate internal from safe user-facing errors.

Use tracing for structured logging. Do not use println!/dbg! for production logging or log credentials.

## 12. Security and External Systems
Never hardcode secrets or plaintext passwords, trust unvalidated input, expose credentials, or log secrets. Validate input; separate authentication from authorization.

Isolate external providers behind adapters/contracts; business logic must not depend on vendor SDKs.

## 13. Testing and Verification
Unit test domain rules/pure functions; integration test repositories, DB boundaries, adapters, and HTTP endpoints; e2e-test important workflows.

Every new API endpoint must have appropriate integration-test coverage. Use Docker-based isolated DB infrastructure for integration tests when practical.

Use repository commands/CI. Typical checks:
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets

Never claim unverified tests, lint, builds, commits, pushes, PRs, or CI checks.

## 13. Testing and Verification
Unit test domain rules/pure functions; integration test repositories, DB boundaries, adapters, and HTTP endpoints; e2e-test important workflows.

Test assertion rule:
- Never assert localized user-facing messages, UI copy, rendered prose, or other human-readable text in ordinary behavioral tests.
- Do not replace one literal locale string with another just to make a test pass.
- Prefer stable behavioral or protocol assertions: HTTP status, redirect location, content type, response envelope/shape, error code, authorization result, persistence state, and other machine-verifiable contract semantics.
- For SSR/page-rendering tests, verify that the page renders successfully and assert structural/protocol properties such as status and content type; do not assert translated titles, labels, descriptions, or body copy.
- Exact message assertions are allowed only when the test's explicit purpose is localization, copy/content rendering, or Problem Details presentation.

Every new API endpoint must have appropriate integration-test coverage. Use Docker-based isolated DB infrastructure for integration tests when practical.

Use repository commands/CI. Typical checks:
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets

Never claim unverified tests, lint, builds, commits, pushes, PRs, or CI checks.

## 14. CLI and Documentation
Prefer Cargo, Git, Docker/Compose, migration tools, and official generators. Present commands in execution order.

Keep docs synchronized. Significant architecture changes should document purpose, data flow, decisions, trade-offs, and operations.

## 15. Source Discipline
Use docs/AI_AGENT_SOURCES.md as the source map. For changing technical behavior, verify repository versions and current official docs. Stale project instructions never override current repository state.

## 16. Engineering Judgment
Challenge unnecessary complexity, coupling, technical debt, security risk, testing difficulty, or non-idiomatic Rust; recommend simpler alternatives. The user decides among valid options.

## 17. Implementation Reporting
For implementation work report Goal, verified Current State, Design Decision, Steps, exact Verification, and GitHub state (repo, branch, files, verification, commit/push/PR status). Never fabricate hashes, PRs, URLs, CI, or test results.

## 18. Definition of Done
Complete only to the level actually achieved:
Requirement understood -> Repository inspected -> Design validated -> Implementation completed -> Relevant tests -> Formatting -> Compilation -> Tests -> Diff review -> GitHub state verification.

Not every task needs every stage; never claim an unperformed stage.

## Final Rule
Inspect the real repository. Understand before changing. Make the smallest correct change. Verify the result. Review the diff. Publish only when explicitly authorized. Keep GitHub as the source of truth.

Every iteration should improve both the software and the engineer's understanding of Rust and systems architecture.