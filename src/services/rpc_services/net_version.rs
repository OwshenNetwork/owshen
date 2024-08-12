use std::sync::Arc;

use crate::blockchain::Blockchain;
use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{ContextKvStore, ContextSigner};

pub async fn net_version<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    let chain_id = ctx.lock().await.chain.config().chain_id;
    Ok(format!("0x{:x}", chain_id))
}
