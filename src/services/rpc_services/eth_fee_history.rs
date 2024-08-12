use std::sync::Arc;

use crate::services::{ContextKvStore, ContextSigner, rpc_services::test_config};
use anyhow::Result;
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

use super::Context;

pub async fn eth_fee_history<S: ContextSigner, K: ContextKvStore>(
    _ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    _params: Params<'static>,
) -> Result<serde_json::Value> {
    let _count = 0;

    Ok(json!({
        "baseFeePerGas": [
            "0",
            "0",
            "0",
            "0",
            "0",
            "0"
        ],
        "gasUsedRatio": [
            0,
            0,
            0,
            0,
            0
        ],
        "oldestBlock": "0xfab8ac",
        "reward": [
            [
                "0",
                "0"
            ],
            [
                "0",
                "0"
            ],
            [
                "0",
                "0"
            ],
            [
                "0",
                "0"
            ],
            [
                "0",
                "0"
            ]
        ]
    }))
}


#[tokio::test]
async fn test_eth_fee_history() {
    let _ctx = test_config().await;
    let params = Params::new(None);
    let result = eth_fee_history(_ctx.into(), params).await;
    match result {
        Ok(value) => {
            assert_eq!(value["baseFeePerGas"], json!(["0", "0", "0", "0", "0", "0"]));
            assert_eq!(value["gasUsedRatio"], json!([0, 0, 0, 0, 0]));
            assert_eq!(value["oldestBlock"], "0xfab8ac");
            assert_eq!(value["reward"], json!([
                ["0", "0"],
                ["0", "0"],
                ["0", "0"],
                ["0", "0"],
                ["0", "0"]
            ]));
        }
        Err(e) => {
            panic!("eth_fee_history failed: {:?}", e);
        }
    }
}
