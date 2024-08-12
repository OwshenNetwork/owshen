use std::sync::Arc;

use alloy::consensus::{TxEip1559, TypedTransaction};
use alloy::network::{Ethereum, EthereumWallet, NetworkWallet};
use alloy::primitives::{Address, Bytes, Uint, B256};
use alloy::rlp::Decodable;
use alloy::rpc::types::{AccessList, AccessListItem};
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

use super::Context;
use crate::blockchain::Blockchain;
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};
use crate::types::OwshenTransaction;

pub async fn eth_send_raw_transaction<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let params: Vec<String> = params.parse().unwrap_or_default();
    let raw_tx = &params[0];
    let raw_tx_bytes = hex::decode(raw_tx.trim_start_matches("0x"))?;
    let mut hah = raw_tx_bytes.as_ref();
    let tx = OwshenTransaction::Eth(alloy::consensus::TxEnvelope::decode(&mut hah).unwrap());
    ctx.lock().await.tx_queue.enqueue(tx.clone());

    Ok("Transaction sent successfully".to_string())
}

#[tokio::test]
async fn test_eth_send_raw_transaction() {
    let _ctx = test_config().await;

    let wallet: EthereumWallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx = TxEip1559 {
        nonce: 0,
        gas_limit: 21_000,
        to: alloy::primitives::TxKind::Call(Address::from([6; 20])),
        value: Uint::<256, 4>::from(0),
        input: Bytes::from("hello"),
        chain_id: _ctx.lock().await.chain.config().chain_id,
        max_priority_fee_per_gas: 3_000_000,
        max_fee_per_gas: 300_000_000,
        access_list: AccessList(vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![B256::ZERO],
        }]),
    };

    let typed_tx = TypedTransaction::Eip1559(tx);
    let signed_tx =
        <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction(&wallet, typed_tx)
            .await
            .unwrap();

    let raw_tx_bytes = alloy::rlp::encode(&signed_tx);
    let raw_tx = hex::encode(raw_tx_bytes);

    let j = json!([format!("0x{}", raw_tx)]);
    let raw_tx_static: &'static str = Box::leak(j.to_string().into_boxed_str());
    let params = Params::new(Some(raw_tx_static));

    let result = eth_send_raw_transaction(_ctx.clone().into(), params).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Transaction sent successfully");

    let ctx = _ctx.lock().await;
    let tx_queue = ctx.tx_queue.queue();
    assert_eq!(tx_queue.len(), 1);
    let queued_tx = tx_queue[0].clone();
    let eth_tx: OwshenTransaction = OwshenTransaction::Eth(signed_tx);

    assert_eq!(queued_tx, eth_tx);
}
