# OpenClaw – Solana HFT Platform

A **Rust-only monorepo** for high-frequency trading on Solana via Triton Dragon's Mouth gRPC, with a focus on Pump.fun token launches.

## Architecture

| Crate | Purpose |
|-------|---------|
| `openclaw-config` | YAML config loader with `${ENV}` variable interpolation |
| `openclaw-telemetry` | Structured tracing + Prometheus metrics exporter |
| `grpc-streamer` | Yellowstone/Dragon's Mouth gRPC subscriber with auto-reconnect |
| `carbon-indexer` | Filters Pump.fun program events from the gRPC stream |
| `hft-bot` | Minimal bot runtime loop scaffold |

## Quick Start

```bash
# 1. Copy and edit environment file
cp .env.example .env

# 2. Build all crates
make build

# 3. Run tests
make test

# 4. Lint + format check
make lint

# 5. Spin up services locally
make up
```

## Configuration

All service configs live in `config/`. Secrets are injected via environment variables (see `.env.example`).

## Next Steps

1. **Integrate official Yellowstone gRPC proto** – replace the stub proto with the official [Yellowstone gRPC protobuf](https://github.com/rpcpool/yellowstone-grpc) definition for `SubscribeRequest`/`SubscribeUpdate`.
2. **Execution engine** – connect `hft-bot` to Triton RPC / Jito block-engine for transaction submission with priority fees.
3. **Signal logic** – implement Pump.fun bonding-curve detection, price-impact calculation, and entry/exit rules in `carbon-indexer` / `hft-bot`.
4. **Observability** – wire Grafana dashboards to the Prometheus endpoint exposed by `openclaw-telemetry`.
5. **Helm / K8s** – add Helm chart for production cluster deployment.

## CI

GitHub Actions runs `cargo build --release`, `cargo test --all`, `cargo clippy`, and `cargo fmt --check` on every push to `main`.
