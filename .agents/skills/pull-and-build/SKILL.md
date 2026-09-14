---
name: pull-and-build
description: >-
  Pulls latest code on the current branch, then builds the workspace and
  iteratively fixes any compile errors, clippy warnings, or test failures until
  the workspace is clean. Use when the user says "pull and build", "sync and
  build", or "update and fix".
---

# Pull & Smart Build — MiniRust

Perform each phase in order. Do not skip a phase. Report results at every step.

---

## Phase 1 — Check working tree

Before pulling, ensure the working tree is safe.

```powershell
# Show current branch and dirty files
git status --short
git branch --show-current
```

**Decision table:**

| Working tree state | Action |
|---|---|
| Clean (no modified files) | Proceed to Phase 2 |
| Unstaged/staged changes | Stash changes: `git stash push -m "pre-pull stash"`, note stash ref |
| Untracked files only | Proceed — untracked files are safe |

> Report the current branch name and any stash performed.

---

## Phase 2 — Pull latest code

```powershell
git pull --ff-only
```

**Handle pull outcomes:**

| Exit code / output | Action |
|---|---|
| Success, "Already up to date" | Continue — nothing changed, still run build |
| Success, files changed | Continue to Phase 3 |
| `CONFLICT` merge error | **Stop.** Report conflicts. Do not attempt to auto-resolve conflicts — ask the user. |
| `! [rejected]` non-fast-forward | Run `git pull --rebase` instead. Report what happened. |
| Auth / network error | **Stop.** Report the error and ask the user to check credentials or network. |

> Report: branch name, commit hash before and after (`git rev-parse --short HEAD`), how many commits pulled.

---

## Phase 3 — Smart Build (iterative fix loop)

Run build and fix errors in a loop. **Maximum 3 fix iterations** to avoid infinite loops.

### 3a. Initial build attempt

```powershell
cargo build --workspace 2>&1
```

If this succeeds with zero errors → jump to Phase 4.

### 3b. Fix loop (repeat up to 3 times if errors remain)

For each iteration:

1. **Parse errors** — read `cargo build` output. Identify:
   - File path (e.g., `src/main.rs:42:5`)
   - Error code (e.g., `E0308`, `E0425`, `E0061`)
   - Error message

2. **Classify and fix** using this priority order:

   | Error type | Fix strategy |
   |---|---|
   | Missing import / `use` | Add the correct `use` path |
   | Type mismatch (`E0308`) | Fix the type at the call site, not by adding casts |
   | Undefined variable / fn (`E0425`, `E0061`) | Fix name or add missing implementation |
   | Borrow / lifetime error | Restructure ownership — avoid adding `clone` unless minimal |
   | Unused import warning-as-error | Remove the `use` statement |
   | Trait not in scope | Add the correct `use Trait;` |
   | `unwrap()` on `Option`/`Result` in service/core | Replace with `?` or `map_err` |

3. **Apply fixes** — edit the affected files following the rules in `.agents/rules/rust-style.md`.

4. **Rebuild** after fixes:
   ```powershell
   cargo build --workspace 2>&1
   ```

5. If build succeeds → break out of loop. If still failing after 3 iterations → go to Phase 3c.

### 3c. Escalation (if 3 iterations exhausted)

- Report all remaining errors with full context (file, line, error code, message).
- List what was tried and why it did not resolve.
- **Do not make further guesses.** Ask the user for guidance.

---

## Phase 4 — Clippy check

```powershell
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1
```

Fix each warning using the same iterative approach (max 2 iterations).

Common quick fixes:
- `clippy::needless_return` → remove `return`
- `clippy::redundant_closure` → use function reference
- `clippy::map_unwrap_or` → use `.unwrap_or_else()`
- `clippy::let_unit_value` → remove the binding

---

## Phase 5 — Tests

```powershell
cargo test --workspace --all-targets 2>&1
```

If tests fail:
1. Report which test(s) failed and the assertion message.
2. Check if the test logic is now stale after the pulled changes.
3. Fix the test if the production behavior changed and the test expectation is wrong.
4. **Do not change production code to make a failing test pass without understanding the failure.**

---

## Phase 6 — Report

Produce a concise summary:

```
## Pull & Build Report

Branch:   <branch-name>
Commits:  <before-hash>..<after-hash>  (<N> new commits)
Stash:    <none | stash@{0}: pre-pull stash>

### Build
- [PASS|FAIL] cargo build --workspace
- Fix iterations: <N>
- Files changed: <list>

### Clippy
- [PASS|FAIL] cargo clippy ...

### Tests
- [PASS|FAIL] cargo test --workspace --all-targets
- Failed tests: <list or none>

### Result
<CLEAN — workspace is ready> or <NEEDS ATTENTION — list blockers>
```

If a stash was created in Phase 1, remind the user: `git stash pop` to restore local changes.

---

## Hard constraints

- Do **not** `git push` unless the user explicitly asks.
- Do **not** modify files outside the workspace.
- Do **not** change business logic to fix a test — fix the test assertion or report to user.
- Do **not** add `#[allow(clippy::...)]` attributes to suppress warnings — fix the code.
- Do **not** auto-resolve merge conflicts — escalate to the user.
