use std::sync::Arc;

use crate::{
    blockchain::Blockchain,
    db::{Key, KvStore, Value},
};
use alloy::primitives::Address;
use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};

pub async fn eth_block_number<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    let height = ctx.lock().await.chain.get_height()?;
    Ok(format!("0x{:x}", height))
}

#[tokio::test]
async fn test_eth_block_number() {
    let _ctx = test_config().await;
    _ctx.lock()
        .await
        .chain
        .db
        .put(Key::Height, Some(Value::Usize(12345)))
        .unwrap();

    let params = Params::new(None);

    let result = eth_block_number(_ctx.into(), params).await.unwrap();

    assert_eq!(result, "0x3039");
}
