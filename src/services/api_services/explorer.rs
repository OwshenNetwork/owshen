use super::Context;
use crate::{
    blockchain::Blockchain,
    db::KvStore,
    services::{ContextKvStore, ContextSigner},
    types::{Block, IncludedTransaction, OwshenTransaction},
};
use alloy::{
    primitives::{FixedBytes, U256},
    signers::Signer,
};
use axum::extract::Json;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTransactionsByBlockRequest {
    pub block_index: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTransactionByHashRequest {
    pub tx_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTransactionByHashResponse {
    pub transaction: IncludedTransaction,
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTransactionByBlockResponse {
    pub transactions: Vec<OwshenTransaction>,
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBlockByHashRequest {
    pub block_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBlockByHashResponse {
    pub block: Block,
    pub success: bool,
}

pub async fn get_transactions_by_block_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    Json(req): Json<GetTransactionsByBlockRequest>,
) -> Result<Json<GetTransactionByBlockResponse>, String>
where
    K: KvStore + Sync + Send + 'static,
{
    let ctx = ctx.lock().await;
    let chain = &ctx.chain;
    match chain.get_transactions_by_block(req.block_index) {
        Ok(txs) => Ok(Json(GetTransactionByBlockResponse {
            success: true,
            transactions: txs,
        })),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn get_transaction_by_hash_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    req: Json<GetTransactionByHashRequest>,
) -> Result<Json<GetTransactionByHashResponse>, String>
where
    K: KvStore + Sync + Send + 'static,
{
    let ctx = ctx.lock().await;
    let tx_hash = match FixedBytes::<32>::from_str(&req.tx_hash) {
        Ok(hash) => hash,
        Err(_) => return Err("Invalid transaction hash".to_string()),
    };
    match ctx.chain.get_transaction_by_hash(tx_hash) {
        Ok(tx) => Ok(Json(GetTransactionByHashResponse {
            success: true,
            transaction: tx,
        })),
        Err(e) => Err(format!("Error: {}", e)),
    }
}
