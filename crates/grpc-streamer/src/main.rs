use anyhow::Result;
use openclaw_config::load;
use openclaw_telemetry::init;
use serde::Deserialize;
use tracing::info;

use grpc_streamer::{streamer::StreamerConfig, Streamer};

#[derive(Deserialize)]
struct Config {
    endpoint: String,
    x_token: String,
    #[serde(default = "default_metrics_addr")]
    metrics_addr: String,
    #[serde(default)]
    account_owners: Vec<String>,
}

fn default_metrics_addr() -> String {
    "0.0.0.0:9090".into()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cfg: Config = load("config/grpc_streamer.yaml")?;
    init(&cfg.metrics_addr)?;
    info!("grpc-streamer starting");

    let streamer_cfg = StreamerConfig {
        endpoint: cfg.endpoint,
        x_token: cfg.x_token,
        ..Default::default()
    };
    let streamer = Streamer::new(streamer_cfg);
    streamer
        .run_forever(cfg.account_owners, |update| {
            info!(?update, "received update");
        })
        .await
}
