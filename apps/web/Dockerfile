FROM node:22-bookworm AS assets
WORKDIR /app
COPY apps/web/package.json apps/web/tailwind.css ./apps/web/
COPY apps/web/src ./apps/web/src
RUN cd apps/web && npm install --no-audit --no-fund && npm run build:css

FROM rust:1.95-bookworm AS builder
WORKDIR /app
RUN cargo install cargo-leptos --version 0.3.6 --locked
RUN rustup target add wasm32-unknown-unknown
COPY . .
COPY --from=assets /app/apps/web/src/generated.css /app/apps/web/src/generated.css
RUN cd apps/web && cargo leptos build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /var/log/minirust
COPY --from=builder /app/target/release/minirust-web /usr/local/bin/minirust-web
COPY --from=builder /app/target/site /app/site
ENV LEPTOS_SITE_ROOT=/app/site
EXPOSE 3001
CMD ["/usr/local/bin/minirust-web"]
