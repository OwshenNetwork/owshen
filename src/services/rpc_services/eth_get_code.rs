use std::sync::Arc;

use anyhow::{Ok, Result};
use jsonrpsee::types::Params;
use tokio::sync::Mutex;

use super::Context;
use crate::services::{ContextKvStore, ContextSigner};

pub async fn eth_get_code<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let params: Vec<String> = params.parse()?;
    let _address = &params[0];

    //TODO: Handle the get code

    let code = "".to_string();

    Ok(code)
}
