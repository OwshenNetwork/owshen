use std::sync::Arc;

use crate::config::{NodeContext, OwshenTransaction};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostTransactRequest {
    pub tx: OwshenTransaction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostTransactResponse {}

pub async fn transact(
    ctx: Arc<Mutex<NodeContext>>,
    req: PostTransactRequest,
) -> Result<Json<PostTransactResponse>, eyre::Report> {
    ctx.lock().await.mempool.push(req.tx);
    Ok(Json(PostTransactResponse {}))
}
