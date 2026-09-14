---
description: Inspect workspace layout, crate roles, and public API surface
---

Show workspace structure:
```bash
cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json; [print(p['name'], p['manifest_path']) for p in json.load(sys.stdin)['packages']]"
```

Then summarize:
- Each crate with its role (from architecture rule).
- Public items in `crates/core` and `crates/services`.
- Any layering violations (framework imports in core/services).
- Any TODO/FIXME comments.
