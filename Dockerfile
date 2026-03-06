FROM rust:1.85 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    nodejs \
    npm \
    python3 \
    tini \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pm2 /usr/local/bin/pm2

RUN mkdir -p /root/.pm2/logs

WORKDIR /app

ENTRYPOINT ["/usr/bin/tini", "--", "pm2"]
CMD ["--help"]

LABEL org.opencontainers.image.title="pm2-rust" \
      org.opencontainers.image.description="A Rust implementation of PM2 process manager" \
      org.opencontainers.image.source="https://github.com/your-repo/pm2-rust"
