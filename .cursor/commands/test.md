---
description: Run only the test suite for fast feedback
---

Full workspace:
```bash
cargo test --workspace --all-targets 2>&1
```

Single crate:
```bash
cargo test -p minirust-<crate> 2>&1
```

Single test with output:
```bash
cargo test -p minirust-<crate> <test_name> -- --nocapture 2>&1
```

Report total tests run, passed, failed. Show full output for failures. No external services required.
