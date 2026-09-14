---
description: Auto-format workspace with rustfmt, then fix clippy warnings
---

Step 1 — Format:
```bash
cargo fmt --all
```

Step 2 — Fix auto-fixable clippy warnings:
```bash
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty -- -D warnings
```

Step 3 — Verify clean:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not suppress warnings with `#[allow(...)]` — fix the code.
