use std::{net::SocketAddr, sync::Arc};

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::{NodeContext, Peer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetHandShakeRequest {
    pub addr: Option<SocketAddr>,
    pub is_client: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetHandShakeResponse {
    pub current_block_number: u64,
}

pub async fn handshake(
    Json(req): Json<GetHandShakeRequest>,
    context: Arc<Mutex<NodeContext>>,
) -> Result<Json<GetHandShakeResponse>, eyre::Report> {
    let mut context = context.lock().await;

    if let Some(addr) = req.addr {
        context.node_manager.add_peer(Peer {
            addr: addr,
            current_block: 0,
        });
    }

    Ok(Json(GetHandShakeResponse {
        current_block_number: context.currnet_block_number,
    }))
}
