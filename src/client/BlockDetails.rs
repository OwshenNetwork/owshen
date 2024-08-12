use crate::blockchain::{Blockchain, Config, Owshenchain};
use crate::config;
use crate::services::{Context, ContextKvStore, ContextSigner}; // Your own services module
use crate::types::OwshenTransaction;
use crate::{db::RamKvStore, genesis::GENESIS};
use anyhow::anyhow;
use axum::response::Html;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tinytemplate::TinyTemplate;
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct BlockDetailsContext {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Clone)]
struct BlockInfo {
    block: String,
    num_txs: u32,
    who: String,
    wen: String,
}
#[derive(Serialize, Clone)]
struct HeightInfo {
    height: usize,
}

#[derive(Serialize)]
struct TemplateContext {
    height_info: HeightInfo,
    block_txs: Vec<OwshenTransaction>,
    time_stamp: u64,
    block_id: String,
    blocks: Vec<BlockInfo>,
}

static TEMPLATE: &str = include_str!("./templates/block_details.html");

pub async fn block_details_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    block_details_ctx: BlockDetailsContext, // Add this to receive block context
) -> Result<Html<String>, anyhow::Error> {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };

    let chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let height_info = match chain.get_height() {
        Ok(height) => HeightInfo { height },
        Err(_) => HeightInfo { height: 0 }, // Default value if get_height fails
    };

    let block_index = usize::from_str(&block_details_ctx.id).unwrap();

    let block_txs = match chain.get_transactions_by_block(block_index) {
        Ok(transactions) => transactions,
        Err(_) => Vec::new(), // Return an empty vector if the operation fails
    };

    let blocks = vec![
        BlockInfo {
            block: "#1431".to_string(),
            num_txs: 189,
            who: "0x8aF3d2E...".to_string(),
            wen: "just now".to_string(),
        },
        BlockInfo {
            block: "#1430".to_string(),
            num_txs: 123,
            who: "0x8aF3d2E...".to_string(),
            wen: "10 sec ago".to_string(),
        },
        BlockInfo {
            block: "#1429".to_string(),
            num_txs: 764,
            who: "0x8aF3d2E...".to_string(),
            wen: "20 sec ago".to_string(),
        },
        BlockInfo {
            block: "#1428".to_string(),
            num_txs: 904,
            who: "0x8aF3d2E...".to_string(),
            wen: "40 sec ago".to_string(),
        },
        BlockInfo {
            block: "#1427".to_string(),
            num_txs: 102,
            who: "0x8aF3d2E...".to_string(),
            wen: "1 min ago".to_string(),
        },
    ];

    let time_stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Initialize TinyTemplate
    let mut tt = TinyTemplate::new();
    tt.add_template("response", TEMPLATE)?;

    // Create a context and insert the blocks, height, and last block information
    let context = TemplateContext {
        height_info,
        block_txs,
        time_stamp,
        block_id: block_details_ctx.id.clone(),
        blocks,
    };

    match tt.render("response", &context) {
        Ok(res) => Ok(Html::from(res)),
        Err(e) => Err(anyhow!(e)),
    }
}
