FROM node:22-bookworm AS assets
WORKDIR /app
COPY apps/web/package.json apps/web/tailwind.css ./apps/web/
COPY apps/web/src ./apps/web/src
RUN cd apps/web && npm install --no-audit --no-fund && npm run build:css

FROM rust:1.90-bookworm AS builder
WORKDIR /app
COPY . .
COPY --from=assets /app/apps/web/src/generated.css /app/apps/web/src/generated.css
RUN cargo build --release -p minirust-web

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/minirust-web /usr/local/bin/minirust-web
EXPOSE 3001
CMD ["/usr/local/bin/minirust-web"]
