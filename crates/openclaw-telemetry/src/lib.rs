//! Telemetry initialisation helpers.
//!
//! Call [`init`] once at process startup to configure:
//! * JSON-structured tracing via `tracing-subscriber`
//! * Prometheus metrics exported on a TCP socket via `metrics-exporter-prometheus`

use anyhow::Result;
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialise tracing and Prometheus metrics.
///
/// `metrics_addr` – socket address to serve `/metrics` on, e.g. `"0.0.0.0:9090"`.
pub fn init(metrics_addr: &str) -> Result<()> {
    init_tracing();
    init_metrics(metrics_addr)?;
    Ok(())
}

/// Set up JSON-structured tracing with `RUST_LOG` env-filter.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().json())
        .init();
}

/// Start the Prometheus metrics exporter on `addr`.
pub fn init_metrics(addr: &str) -> Result<()> {
    let socket_addr: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid metrics address `{}`: {}", addr, e))?;
    PrometheusBuilder::new()
        .with_http_listener(socket_addr)
        .install()
        .map_err(|e| anyhow::anyhow!("failed to install Prometheus exporter: {}", e))?;
    tracing::info!(addr = %addr, "Prometheus metrics exporter started");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_addr_returns_error() {
        let result = init_metrics("not-an-address");
        assert!(result.is_err());
    }
}
