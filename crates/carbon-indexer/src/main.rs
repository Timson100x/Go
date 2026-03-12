use anyhow::Result;
use openclaw_config::load;
use openclaw_telemetry::init;
use serde::Deserialize;
use tracing::info;

use grpc_streamer::{proto::subscribe_update::UpdateOneof, streamer::StreamerConfig, Streamer};

#[derive(Deserialize)]
struct Config {
    endpoint: String,
    x_token: String,
    pumpfun_program_id: String,
    #[serde(default = "default_metrics_addr")]
    metrics_addr: String,
}

fn default_metrics_addr() -> String {
    "0.0.0.0:9091".into()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cfg: Config = load("config/carbon.yaml")?;
    init(&cfg.metrics_addr)?;

    let program_id = cfg.pumpfun_program_id.clone();
    info!(program_id = %program_id, "carbon-indexer starting – filtering Pump.fun events");

    let streamer_cfg = StreamerConfig {
        endpoint: cfg.endpoint,
        x_token: cfg.x_token,
        ..Default::default()
    };

    let streamer = Streamer::new(streamer_cfg);
    let owners = vec![program_id.clone()];

    streamer
        .run_forever(owners, move |update| {
            if let Some(UpdateOneof::Transaction(tx)) = update.update_oneof {
                info!(
                    signature = %tx.signature,
                    slot = tx.slot,
                    "Pump.fun transaction detected"
                );
                metrics::counter!("carbon_indexer.pumpfun_tx_total").increment(1);
            }
        })
        .await
}
