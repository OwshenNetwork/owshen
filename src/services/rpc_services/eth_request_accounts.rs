use std::sync::Arc;

use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{ContextKvStore, ContextSigner};

pub async fn eth_request_accounts<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    //TODO: Handle the request accounts
    let accounts = "".to_string();
    Ok(accounts)
}
