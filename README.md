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

### Configuration

The web application supports the following environment variables:

- `MINIRUST_HOST` — server host; defaults to `127.0.0.1`.
- `MINIRUST_PORT` — server port; defaults to `3000`.

For example:

```bash
MINIRUST_HOST=0.0.0.0 MINIRUST_PORT=8080 cargo run -p minirust-web
```

Press `Ctrl+C` to shut down the server gracefully.
