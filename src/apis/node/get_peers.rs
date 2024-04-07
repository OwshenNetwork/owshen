use std::sync::Arc;

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::{NodeContext, Peer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPeersResponse {
    pub peers: Vec<Peer>,
}

pub async fn get_peers(
    context: Arc<Mutex<NodeContext>>,
) -> Result<Json<GetPeersResponse>, eyre::Report> {
    let context = context.lock().await;
    Ok(Json(GetPeersResponse {
        peers: context.node_manager.get_peers().clone(),
    }))
}
