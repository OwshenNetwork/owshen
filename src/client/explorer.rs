use std::sync::Arc;
use tokio::sync::Mutex;

use crate::blockchain::Blockchain;
use crate::services::{ContextKvStore, ContextSigner};
use crate::types::Block;
use anyhow::anyhow;
use axum::response::Html;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tinytemplate::TinyTemplate;

use crate::services::Context;

#[derive(Serialize, Clone)]
pub struct ExplorerContext {
    pub name: String,
}

#[derive(Serialize, Clone)]
struct HeightInfo {
    height: usize,
}

#[derive(Serialize)]
struct TemplateContext {
    height_info: HeightInfo,
    last_blocks: Vec<Block>,
    time_stamp: u64,
}

static TEMPLATE: &str = include_str!("./templates/explorer.html");

pub async fn explorer_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
) -> Result<Html<String>, anyhow::Error> {
    let mut _chain = &ctx.lock().await.chain;

    // Fetch the height information
    let height_info = match _chain.get_height() {
        Result::Ok(height) => HeightInfo { height },
        Result::Err(_) => HeightInfo { height: 0 }, // Default value if get_height fails
    };

    let last_blocks_result = _chain.get_blocks(0, 10);
    let last_blocks = match last_blocks_result {
        Ok(block_opt) => block_opt,
        Err(_) => Vec::new(), // Return an empty vector instead of doing nothing
    };
    let time_stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut tt = TinyTemplate::new();
    tt.add_template("response", TEMPLATE)?;
    let context = TemplateContext {
        height_info,
        last_blocks,
        time_stamp,
    };

    match tt.render("response", &context) {
        Ok(res) => Ok(Html::from(res)),
        Err(e) => Err(anyhow!(e)),
    }
}
