use std::{net::SocketAddr, sync::Arc};

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::{NodeContext, Peer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub up: bool,
    pub peers: Vec<Peer>,
    pub current_block_number: u64,
    pub addr: SocketAddr,
}

pub async fn status(
    context: Arc<Mutex<NodeContext>>,
) -> Result<Json<GetStatusResponse>, eyre::Report> {
    let context = context.lock().await;

    Ok(Json(GetStatusResponse {
        up: true,
        peers: context.node_manager.peers.clone(),
        current_block_number: context.currnet_block_number,
        addr: context
            .node_manager
            .external_addr
            .clone()
            .ok_or(eyre::eyre!("Not a node!"))?,
    }))
}
