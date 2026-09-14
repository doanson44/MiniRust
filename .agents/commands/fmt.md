---
description: Auto-format the entire workspace with rustfmt, then fix any remaining clippy warnings.
---

Run from the workspace root.

Step 1 — Format all source files:
```powershell
cargo fmt --all
```

Step 2 — Run clippy and fix auto-fixable warnings:
```powershell
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty -- -D warnings
```

Step 3 — Verify no remaining warnings:
```powershell
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Report which files were reformatted and which warnings were fixed. Do not suppress warnings with `#[allow(...)]` — fix the code.
