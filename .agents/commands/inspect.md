---
description: Inspect the current workspace layout — members, crate sizes, public API surface.
---

Run from the workspace root:

```powershell
# List all workspace members
cargo metadata --no-deps --format-version 1 | ConvertFrom-Json | Select-Object -ExpandProperty packages | Select-Object name, version, manifest_path

# Show file tree (exclude target/)
Get-ChildItem -Recurse -File | Where-Object { $_.FullName -notmatch '\\target\\' -and $_.FullName -notmatch '\\.git\\' } | Select-Object FullName, Length | Sort-Object FullName
```

Then summarize:
- List each crate with its role (from the architecture rule).
- List all public `pub` items in `crates/core` and `crates/services`.
- Note any deviations from the layering rule (e.g., framework imports in core/services).
- Flag any TODO/FIXME comments in source files.
