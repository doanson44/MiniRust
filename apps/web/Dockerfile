FROM rust:1.90-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p minirust-web

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/minirust-web /usr/local/bin/minirust-web
EXPOSE 3001
CMD ["/usr/local/bin/minirust-web"]
