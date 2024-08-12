use std::sync::Arc;

use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{ContextKvStore, ContextSigner};

pub async fn eth_get_gas_price<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    let gas_price = 0x0;

    Ok(format!("0x{:x}", gas_price))
}
