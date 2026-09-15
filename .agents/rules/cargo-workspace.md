---
description: Cargo workspace and dependency conventions. Applied when editing Cargo.toml files.
trigger: model_decision
globs: ["**/Cargo.toml"]
---

# Cargo Workspace — MiniRust

## Structure

- Root `Cargo.toml` declares all workspace members and is the single source of shared dependency versions.
- Packages use `version.workspace = true`, `edition.workspace = true`, and `license.workspace = true`.
- Package names follow `minirust-<name>`.

## Architecture rule

The workspace is organized around transport apps, domain primitives, application/CQRS logic, and infrastructure adapters. Do not turn the workspace into a collection of unrelated global service crates.

## Adding a dependency

1. Add the version to `[workspace.dependencies]` in the root `Cargo.toml`.
2. Reference it from a package with `dep.workspace = true`.
3. Keep infrastructure dependencies out of domain code.

## Adding a crate

- Apps: `apps/<name>/` → `minirust-<name>`
- Libraries: `crates/<name>/` → `minirust-<name>`
- Add the new path to workspace `members`.
- Create a crate only when a real feature or architectural boundary requires it.

## Current technology constraints

- MariaDB through SQLx's `mysql` driver is the database standard.
- Tailwind CSS is the only frontend CSS framework.
- Do not introduce Bootstrap compatibility.
- Do not introduce Redis or other cache infrastructure.
- Do not introduce unrelated infrastructure dependencies without an explicit feature requirement.

## CI compliance

`Cargo.lock` must remain consistent because CI uses `--locked`.

After changing dependencies, run:

```bash
cargo check --workspace --locked
```
