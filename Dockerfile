FROM rust:1.75-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build --release --bin ice-data-api
RUN cargo build --release --bin ice-data-mcp

FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ice-data-api /usr/local/bin/
EXPOSE 8080
CMD ["ice-data-api"]

FROM debian:bookworm-slim AS mcp
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ice-data-mcp /usr/local/bin/
EXPOSE 3001
CMD ["ice-data-mcp"]
