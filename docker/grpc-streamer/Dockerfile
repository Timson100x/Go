FROM rust:1.75-slim AS builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml .
COPY crates/openclaw-config   crates/openclaw-config
COPY crates/openclaw-telemetry crates/openclaw-telemetry
COPY crates/grpc-streamer      crates/grpc-streamer
COPY crates/carbon-indexer     crates/carbon-indexer
COPY crates/hft-bot            crates/hft-bot

RUN cargo build --release --bin grpc-streamer

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/grpc-streamer /usr/local/bin/grpc-streamer
COPY config /config
ENV RUST_LOG=info
EXPOSE 9090
ENTRYPOINT ["grpc-streamer"]
