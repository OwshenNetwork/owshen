use std::sync::Arc;

use super::Context;
use crate::services::{ContextKvStore, ContextSigner};

use anyhow::Result;
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

pub async fn todo<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<String> {
    Ok("0x0".into())
}
