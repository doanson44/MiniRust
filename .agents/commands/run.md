---
description: Run the API and/or web servers locally for smoke testing. Stop them when done.
---

Start only what is needed. Do not leave processes running unless the user asks.

**Start API server** (port 3000):
```powershell
cargo run -p minirust-api
```
Smoke test:
- `curl http://127.0.0.1:3000/health` → `{"status":"ok","database":"..."}`
- `curl http://127.0.0.1:3000/api/v1/hello` → `{"message":"Hello from MiniRust"}`
- `curl -X POST http://127.0.0.1:3000/api/v1/echo -H "Content-Type: application/json" -d '{"message":"test"}'` → `{"echo":"test"}`
- Browser: `http://127.0.0.1:3000/swagger` → Swagger UI

**Start Web server** (port 3001):
```powershell
cargo run -p minirust-web
```
Smoke test:
- `curl http://127.0.0.1:3001/` → HTML with `<!DOCTYPE html>` and `MiniRust`
- `curl http://127.0.0.1:3001/health` → `ok`

MariaDB is optional — the API binary starts without `MINIRUST_DATABASE_URL`. If set, `/health` also checks DB connectivity.

Stop all processes when smoke testing is complete.
