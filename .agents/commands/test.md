---
description: Run only the test suite — fast feedback without a full build.
---

Run from the workspace root:

```powershell
cargo test --workspace --all-targets 2>&1
```

For a single crate:
```powershell
cargo test -p minirust-<crate> 2>&1
```

For a single test with output visible:
```powershell
cargo test -p minirust-<crate> <test_name> -- --nocapture 2>&1
```

Report: total tests run, passed, failed. Show full output for any failure.

Do not start MariaDB or Redis. Tests must not require external services.
