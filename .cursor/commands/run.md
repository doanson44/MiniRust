---
description: Run servers locally for smoke testing
---

**API** (port 3000): `cargo run -p minirust-api`
- GET /health → `{"status":"ok","database":"..."}`
- GET /api/v1/hello → `{"message":"Hello from MiniRust"}`
- POST /api/v1/echo body `{"message":"test"}` → `{"echo":"test"}`
- Browser /swagger → Swagger UI

**Web** (port 3001): `cargo run -p minirust-web`
- GET / → HTML with `<!DOCTYPE html>` and `MiniRust`
- GET /health → `ok`

MariaDB connection comes from the `MINIRUST_DB_*` variables (`MINIRUST_DATABASE_URL` overrides them when set).

Stop all processes when done.
