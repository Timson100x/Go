//! Reconnect-ready gRPC streamer.

use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;
use tokio_stream::StreamExt as _;
use tonic::transport::Channel;
use tracing::{error, info, warn};

use crate::proto::{
    geyser_client::GeyserClient, SubscribeRequest, SubscribeRequestFilterTransactions,
};

/// Configuration for the gRPC streamer.
#[derive(Debug, Clone)]
pub struct StreamerConfig {
    /// Triton Dragon's Mouth endpoint (e.g. `https://my-node:10000`).
    pub endpoint: String,
    /// Access token sent as `x-token` metadata on every RPC.
    pub x_token: String,
    /// Initial reconnect back-off (doubles on each retry up to [`Self::max_backoff`]).
    pub initial_backoff: Duration,
    /// Maximum reconnect back-off.
    pub max_backoff: Duration,
}

impl Default for StreamerConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://127.0.0.1:10000".into(),
            x_token: String::new(),
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }
}

/// A subscription handle that wraps the gRPC channel and handles reconnects.
pub struct Streamer {
    cfg: StreamerConfig,
}

impl Streamer {
    /// Create a new streamer with the given configuration.
    pub fn new(cfg: StreamerConfig) -> Self {
        Self { cfg }
    }

    /// Connect and return a live gRPC client.
    pub async fn connect(&self) -> Result<GeyserClient<Channel>> {
        let channel = Channel::from_shared(self.cfg.endpoint.clone())
            .context("invalid gRPC endpoint URI")?
            .connect()
            .await
            .context("failed to connect to gRPC endpoint")?;
        Ok(GeyserClient::new(channel))
    }

    /// Run the subscribe loop with automatic reconnect.
    ///
    /// `on_update` is called for every received [`crate::proto::SubscribeUpdate`].
    pub async fn run_forever<F>(&self, account_owners: Vec<String>, mut on_update: F) -> !
    where
        F: FnMut(crate::proto::SubscribeUpdate) + Send,
    {
        let mut backoff = self.cfg.initial_backoff;
        loop {
            info!(endpoint = %self.cfg.endpoint, "connecting to Dragon's Mouth gRPC");
            match self
                .subscribe_once(account_owners.clone(), &mut on_update)
                .await
            {
                Ok(()) => {
                    warn!("gRPC stream ended, reconnecting…");
                }
                Err(e) => {
                    error!(error = %e, backoff_secs = backoff.as_secs(), "gRPC stream error");
                }
            }
            sleep(backoff).await;
            backoff = (backoff * 2).min(self.cfg.max_backoff);
        }
    }

    async fn subscribe_once<F>(&self, account_owners: Vec<String>, on_update: &mut F) -> Result<()>
    where
        F: FnMut(crate::proto::SubscribeUpdate),
    {
        let mut client = self.connect().await?;

        let tx_filter = SubscribeRequestFilterTransactions {
            account_include: account_owners,
            ..Default::default()
        };

        let mut filters = std::collections::HashMap::new();
        filters.insert("openclaw".to_string(), tx_filter);

        let request = SubscribeRequest {
            transactions: filters,
            ..Default::default()
        };

        let outbound = tokio_stream::iter(vec![request]);
        let mut inbound = client
            .subscribe(outbound)
            .await
            .context("subscribe RPC failed")?
            .into_inner();

        while let Some(msg) = inbound.next().await {
            let update = msg.context("stream error")?;
            on_update(update);
        }
        Ok(())
    }
}
