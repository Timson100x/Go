use std::time::Duration;

use anyhow::Result;
use openclaw_config::load;
use openclaw_telemetry::init;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_metrics_addr")]
    metrics_addr: String,
    /// Interval between bot loop ticks in milliseconds.
    #[serde(default = "default_tick_ms")]
    tick_ms: u64,
    /// Dry-run mode – log orders but do not submit them.
    #[serde(default = "default_dry_run")]
    dry_run: bool,
}

fn default_metrics_addr() -> String {
    "0.0.0.0:9092".into()
}

fn default_tick_ms() -> u64 {
    100
}

fn default_dry_run() -> bool {
    true
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cfg: Config = load("config/bot.yaml")?;
    init(&cfg.metrics_addr)?;

    info!(
        dry_run = cfg.dry_run,
        tick_ms = cfg.tick_ms,
        "hft-bot starting"
    );

    run_loop(cfg).await
}

async fn run_loop(cfg: Config) -> Result<()> {
    let tick = Duration::from_millis(cfg.tick_ms);
    let mut interval = tokio::time::interval(tick);

    loop {
        interval.tick().await;
        on_tick(cfg.dry_run).await;
    }
}

async fn on_tick(dry_run: bool) {
    // TODO: implement signal evaluation, position sizing, and order submission.
    // For now just increment the tick counter so the loop is observable.
    metrics::counter!("hft_bot.ticks_total").increment(1);
    if dry_run {
        tracing::debug!("dry-run tick – no orders submitted");
    }
}
