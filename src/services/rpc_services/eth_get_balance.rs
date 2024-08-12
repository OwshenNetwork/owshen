use std::borrow::Cow;
use std::sync::Arc;

use crate::blockchain::Blockchain;
use crate::db::{Key, KvStore, Value};
use alloy::primitives::utils::parse_units;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use anyhow::{anyhow, Result};
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};
use crate::types::Token;

pub async fn eth_get_balance<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let params: Vec<String> = params.parse()?;
    let addr: Address = params
        .get(0)
        .ok_or(anyhow!("Address unavailable!"))?
        .parse()?;
    let balance = ctx.lock().await.chain.get_balance(Token::Native, addr)?;
    Ok(format!("0x{:x}", balance))
}

#[tokio::test]
async fn test_eth_get_balance() {
    let _ctx = test_config().await;

    let address: Address = Address::from([2; 20]);
    let amount = parse_units("2", 18).unwrap().into();

    _ctx.lock()
        .await
        .chain
        .db
        .put(
            Key::Balance(address, Token::Native),
            Some(Value::U256(amount)),
        )
        .unwrap();

    let addr = address.to_string();
    let j = json!([addr, "latest"]).to_string();
    let addr_static: &'static str = Box::leak(j.into_boxed_str());
    let params = Params::new(Some(addr_static));

    let result = eth_get_balance(_ctx.into(), params).await;

    assert!(result.is_ok());

    let balance = result.unwrap();
    assert_eq!(balance, format!("0x{:x}", amount));
}
