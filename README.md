# MiniRust

A minimal Rust project template for learning and experimenting.

## Workspace

This repository is a Cargo workspace with one application:

- `apps/web` — minimal Axum HTTP server

## Development

From the repository root:

```bash
cargo check --workspace
cargo run -p minirust-web
```

The web application listens on `http://127.0.0.1:3000` by default. `GET /` returns a plain-text response.
