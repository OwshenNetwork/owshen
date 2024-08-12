use std::sync::Arc;

use alloy::primitives::{Address, Uint, U256};
use anyhow::{anyhow, Result};
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

use super::Context;
use crate::{
    blockchain::Blockchain,
    db::{Key, KvStore, Value},
    services::{rpc_services::test_config, ContextKvStore, ContextSigner},
};

pub async fn eth_get_transaction_count<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<Uint<256, 4>, anyhow::Error> {
    let params: Vec<String> = params.parse()?;
    let addr: Address = params
        .get(0)
        .ok_or(anyhow!("Address unavailable!"))?
        .parse()?;

    let nonce = ctx.lock().await.chain.get_eth_nonce(addr).unwrap();

    Ok(nonce)
}

#[tokio::test]
async fn test_eth_get_transaction_count() {
    let _ctx = test_config().await;

    let address: Address = Address::from([2; 20]);

    _ctx.lock()
        .await
        .chain
        .db
        .put(Key::NonceEth(address), Some(Value::U256(U256::from(10))))
        .unwrap();

    let addr = address.to_string();
    let j = json!([addr, "latest"]).to_string();
    let addr_static: &'static str = Box::leak(j.into_boxed_str());
    let params = Params::new(Some(addr_static));

    let result = eth_get_transaction_count(_ctx.into(), params).await;

    assert!(result.is_ok());

    let nonce = result.unwrap();
    assert_eq!(nonce, U256::from(10));
}
