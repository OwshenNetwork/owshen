use alloy::{
    network::{Ethereum, Network},
    primitives::{fixed_bytes, keccak256},
    rpc::types::Block,
    signers::{local::PrivateKeySigner, Signer},
};
use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::Context;
use crate::{
    blockchain::{tx::owshen_airdrop::babyjubjub::PrivateKey, Blockchain},
    db::{Key, KvStore, Value},
    services::{rpc_services::test_config, ContextKvStore, ContextSigner},
    types,
};

use jsonrpsee::types::Params;
use tokio::sync::Mutex;

pub async fn eth_get_block_by_number<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<serde_json::Value> {
    let params: Vec<String> = params.parse()?;
    let index: usize = params
        .get(0)
        .ok_or(anyhow!("Address unavailable!"))?
        .parse()?;
    let block = _ctx.lock().await.chain.get_block(index)?;

    Ok(serde_json::json!(block))
}

#[tokio::test]
async fn test_eth_get_block_by_number() {
    let _ctx = test_config().await;

    let block_number: usize = 1234;
    let signer = PrivateKeySigner::random();
    let message = b"hello";
    let signature = signer.sign_message(message).await.unwrap();
    let prev_block_hash = keccak256(vec![1,2,3]);

    let block: types::Block = types::Block {
        prev_hash: prev_block_hash
            .into(),
        index: block_number,
        txs: Vec::new(),
        sig: Some(signature.clone()),
        timestamp: 32,
    };

    _ctx.lock()
        .await
        .chain
        .db
        .put(Key::Block(block_number), Some(Value::Block(block.clone())))
        .unwrap();

    let _ = _ctx
        .lock()
        .await
        .chain
        .db
        .put(Key::Height, Some(Value::Usize(1235)));


    let block_number_str = block_number.to_string();
    let j = serde_json::json!([block_number_str, "latest"]).to_string();
    let addr_static: &'static str = Box::leak(j.into_boxed_str());
    let params = Params::new(Some(addr_static));

    let result = eth_get_block_by_number(_ctx.into(), params).await;


    assert!(result.is_ok());

    let result_block = result.unwrap();
    let expected_block_json = serde_json::json!({
        "prev_hash": prev_block_hash.to_string(),
        "index": block_number,
        "txs": [],
        "sig": {
            "r": format!("0x{:x}", signature.r()),
            "s": format!("0x{:x}", signature.s()),
            "yParity": format!("0x{:x}", signature.v().to_u64()),
        },
        "timestamp": block.timestamp,
    });

    assert_eq!(result_block, expected_block_json);
}
