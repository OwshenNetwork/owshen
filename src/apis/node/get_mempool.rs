use std::sync::Arc;

use crate::config::{NodeContext, OwshenTransaction};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMempoolRequest {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMempoolResponse {
    mempool: Vec<OwshenTransaction>,
}

pub async fn mempool(
    ctx: Arc<Mutex<NodeContext>>,
    _req: GetMempoolRequest,
) -> Result<Json<GetMempoolResponse>, eyre::Report> {
    Ok(Json(GetMempoolResponse {
        mempool: ctx.lock().await.mempool.clone(),
    }))
}
