use std::sync::Arc;

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::{NodeContext, Peer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetHandShakeRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,

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

    if let Some(ip) = req.ip {
        if let Some(port) = req.port {
            context.node_manager.add_peer(Peer { ip, port, current_block: 0});
        }
    }

    Ok(Json(GetHandShakeResponse {
        current_block_number: context.currnet_block_number,
    }))
}
