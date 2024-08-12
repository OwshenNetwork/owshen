use super::Context;

mod net_version;
pub use net_version::*;
mod eth_get_balance;
pub use eth_get_balance::*;
mod eth_chain_id;
pub use eth_chain_id::*;
mod eth_block_number;
pub use eth_block_number::*;
mod eth_call;
pub use eth_call::*;
mod eth_get_code;
pub use eth_get_code::*;
mod eth_request_accounts;
pub use eth_request_accounts::*;
mod eth_estimate_gas;
pub use eth_estimate_gas::*;
mod eth_get_gas_price;
pub use eth_get_gas_price::*;
mod eth_get_transaction_count;
pub use eth_get_transaction_count::*;
mod eth_get_transaction_receipt;
pub use eth_get_transaction_receipt::*;
mod eth_send_raw_transaction;
pub use eth_send_raw_transaction::*;
mod eth_get_block_by_number;
pub use eth_get_block_by_number::*;
mod eth_fee_history;
pub use eth_fee_history::*;
mod eth_get_transaction_by_hash;
pub use eth_get_transaction_by_hash::*;
mod todo;
pub use todo::*;


use alloy::{
    primitives::{utils::parse_units, Address, FixedBytes, Signature, U256},
    signers::{local::PrivateKeySigner, Signer},
};
use axum::{
    body::{Body, HttpBody},
    http::{self, Request, StatusCode},
    routing::get,
    Json, Router,
};
use serde::de::Expected;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::{Service, ServiceExt};

use crate::{
    blockchain::{
        tx::owshen_airdrop::babyjubjub::PrivateKey, Blockchain, Config, Owshenchain,
        TransactionQueue,
    },
    config,
    db::{DiskKvStore, Key, KvStore, RamKvStore, Value},
    genesis::GENESIS,
    safe_signer::{self, SafeSigner},
    services::{api_services::api_routes},
    types::{
        network::Network, Burn, CustomTx, CustomTxMsg, IncludedTransaction, OwshenTransaction,
        Token,
    },
};

async fn test_config() -> Arc<tokio::sync::Mutex<Context<SafeSigner, RamKvStore>>> {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address:  "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };

    let owner = SafeSigner::new(PrivateKeySigner::random());
    return  Arc::new(Mutex::new(Context {
        signer: owner.clone(),
        exit: false,
        tx_queue: TransactionQueue::new(),
        chain: Owshenchain::new(conf, RamKvStore::new()),
    }));
}

