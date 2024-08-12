use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Result};
use jsonrpsee::types::Params;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::Context;
use crate::blockchain::Blockchain;
use crate::db::{Key, KvStore, Value};
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};
use crate::types::network::Network;
use alloy::consensus::{Transaction, TxEip1559, TypedTransaction};
use alloy::hex::ToHexExt;
use alloy::network::{Ethereum, EthereumWallet, NetworkWallet};
use alloy::primitives::{Address, Bytes, FixedBytes, Uint, B256, U256};
use alloy::rpc::types::{AccessList, AccessListItem};
use std::result;

use crate::types::{
    BincodableOwshenTransaction, Burn, CustomTx, CustomTxMsg, IncludedTransaction,
    OwshenTransaction, Token,
};

pub async fn eth_get_transaction_receipt<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let params: Vec<String> = params.parse().unwrap_or_default();

    let tx_hash = match FixedBytes::<32>::from_str(&params[0]) {
        Ok(hash) => hash,
        Err(_) => return Err(anyhow::Error::msg("Invalid transaction hash")),
    };

    let tx = ctx
        .lock()
        .await
        .chain
        .get_transaction_by_hash(tx_hash)
        .unwrap();

    let owshen_tx: OwshenTransaction = tx.tx.try_into()?;

    let (gas_used, effective_gas_price, sender, to, contract_address) = match owshen_tx {
        OwshenTransaction::Eth(tx) => {
            let gas_used = tx.gas_limit();
            let effective_gas_price = tx.as_eip1559().unwrap().tx().effective_gas_price(None);
            let sender = tx.recover_signer()?.to_string();
            let to = match tx.to() {
                alloy::primitives::TxKind::Create => serde_json::Value::Null,
                alloy::primitives::TxKind::Call(address) => {
                    serde_json::Value::String(address.to_string())
                }
            };
            let contract_address = serde_json::Value::Null;
            (gas_used, effective_gas_price, sender, to, contract_address)
        }
        OwshenTransaction::Custom(_) => {
            return Err(anyhow::Error::msg("Not an Ethereum transaction!"))
        }
    };

    let receipt_json = json!({
        "transactionHash": format!("0x{:x}", tx_hash),
        "transactionIndex": format!("0x{:x}", tx.transaction_index),
        "blockHash": format!("0x{:x}", tx.block_hash),
        "blockNumber": format!("0x{:x}", tx.block_number),
        "gasUsed": format!("0x{:x}", gas_used),
        "effectiveGasPrice": format!("0x{:x}", effective_gas_price),
        "from": sender,
        "to": to,
        "contractAddress": contract_address,
        "logs": [],
        "cumulativeGasUsed": "0x1b4",
        "status": "0x1",
        "logsBloom": "0x".to_string() + &"0".repeat(512),
        "type": "0x2",
    })
    .to_string();

    Ok(receipt_json)
}

#[tokio::test]
async fn test_eth_get_transaction_receipt() {
    let _ctx = test_config().await;

    let wallet: EthereumWallet = EthereumWallet::new(PrivateKeySigner::random());
    let msg_sender: Address =
        <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

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

    let eth_tx = OwshenTransaction::Eth(signed_tx.clone());

    let tx_hash = eth_tx.hash().unwrap();

    let included_tx: IncludedTransaction = IncludedTransaction {
        tx: eth_tx.try_into().unwrap(),
        block_hash: FixedBytes::from([0u8; 32]),
        block_number: 4321,
        transaction_index: 1,
    };

    let _ = _ctx.lock().await.chain.db.put(
        Key::TransactionHash(tx_hash),
        Some(Value::Transaction(included_tx.clone())),
    );

    let tx_hash_str = tx_hash.to_string();
    let j = json!([tx_hash_str, "latest"]).to_string();
    let hash_static: &'static str = Box::leak(j.into_boxed_str());
    let params = Params::new(Some(hash_static));

    let result = eth_get_transaction_receipt(_ctx.into(), params).await;

    assert!(result.is_ok());

    let receipt_json: serde_json::Value = serde_json::from_str(result.as_ref().unwrap()).unwrap();

    let expected_json = json!({
        "transactionHash": tx_hash_str,
        "transactionIndex":  format!("0x{:x}",included_tx.transaction_index),
        "blockHash": format!("0x{:x}", included_tx.block_hash),
        "blockNumber": format!("0x{:x}", included_tx.block_number),
        "gasUsed":  format!("0x{:x}",21000),
        "effectiveGasPrice": format!("0x{:x}",300000000),
        "from": msg_sender.to_string(),
        "to": Address::from([6; 20]).to_string(),
        "contractAddress": serde_json::Value::Null,
        "logs": [],
        "cumulativeGasUsed": "0x1b4",
        "status": "0x1",
        "logsBloom": "0x".to_string() + &"0".repeat(512),
        "type": "0x2",
    });

    assert_eq!(receipt_json, expected_json);
}
