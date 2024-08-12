use std::sync::Arc;

use crate::blockchain::Blockchain;
use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};

pub async fn eth_chain_id<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    let chain_id = ctx.lock().await.chain.config().chain_id;
    Ok(format!("0x{:x}", chain_id))
}

#[tokio::test]
async fn test_eth_chain_id() {
    let _ctx = test_config().await;

    let params = Params::new(None);

    let result = eth_chain_id(_ctx.clone().into(), params).await.unwrap();

    assert_eq!(
        result,
        format!("0x{:x}", _ctx.lock().await.chain.config().chain_id)
    );
}
