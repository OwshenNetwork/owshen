use std::sync::Arc;

use anyhow::Result;
use jsonrpsee::types::Params;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use super::Context;
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};

pub async fn eth_estimate_gas<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let params: Vec<String> = params.parse()?;
    let data = params.get(0).map(|s| s.to_owned()).unwrap_or_default();
    let mut gas_estimate = 0u64;

    if !data.is_empty() {
        gas_estimate += (data.len() as u64) * 68;
    }

    Ok(format!("0x{:x}", gas_estimate))
}

#[tokio::test]
async fn test_eth_estimate_gas() {
    let _ctx = test_config().await;

    let params = Params::new(Some("[]"));
    let result = eth_estimate_gas(_ctx.clone().into(), params).await.unwrap();
    assert_eq!(result, "0x0");

}
