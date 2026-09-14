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

MariaDB is optional — binary starts without `MINIRUST_DATABASE_URL`.

Stop all processes when done.
