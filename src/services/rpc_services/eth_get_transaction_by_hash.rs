use std::result;
use std::sync::Arc;

use crate::blockchain::Blockchain;
use crate::db::{Key, KvStore, Value};
use crate::services::{rpc_services::test_config, ContextKvStore, ContextSigner};
use crate::types::network::Network;
use alloy::consensus::{TxEip1559, TypedTransaction};
use alloy::hex::ToHexExt;
use alloy::network::{Ethereum, EthereumWallet, NetworkWallet};
use alloy::primitives::{Address, Bytes, FixedBytes, Uint, B256, U256};
use alloy::rpc::types::{AccessList, AccessListItem};
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Result};
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

use super::Context;
use crate::types::{
    BincodableOwshenTransaction, Burn, CustomTx, CustomTxMsg, IncludedTransaction,
    OwshenTransaction, Token,
};

pub async fn eth_get_transaction_by_hash<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<serde_json::Value> {
    let params: Vec<String> = params.parse().unwrap_or_default();
    let tx_hash = params
        .get(0)
        .ok_or(anyhow!("Transaction hash not provided!"))?;
    let inc_tx = ctx
        .lock()
        .await
        .chain
        .get_transaction_by_hash(tx_hash.parse()?)?;

    match inc_tx.tx.try_into()? {
        OwshenTransaction::Custom(_) => Err(anyhow!("Not a eth transaction!")),
        OwshenTransaction::Eth(envelope) => {
            let mut tx = serde_json::to_value(&envelope)?;
            let v = envelope
                .as_eip1559()
                .ok_or(anyhow!("Only EIP-1559 supported for now!"))?
                .signature()
                .v()
                .to_u64();
            tx.as_object_mut().unwrap().insert(
                "from".into(),
                envelope
                    .recover_signer()?
                    .encode_hex_upper_with_prefix()
                    .into(),
            );
            tx.as_object_mut().unwrap().insert(
                "blockHash".into(),
                inc_tx.block_hash.encode_hex_with_prefix().into(),
            );
            tx.as_object_mut().unwrap().insert(
                "blockNumber".into(),
                format!("0x{:x}", inc_tx.block_number).into(),
            );
            tx.as_object_mut().unwrap().insert(
                "transactionIndex".into(),
                format!("0x{:x}", inc_tx.transaction_index).into(),
            );
            tx.as_object_mut()
                .unwrap()
                .insert("gas".into(), format!("0x{:x}", 0).into());
            tx.as_object_mut()
                .unwrap()
                .insert("gasPrice".into(), format!("0x{:x}", 0).into());
            tx.as_object_mut().unwrap().remove(&"gasLimit".to_string());
            tx.as_object_mut()
                .unwrap()
                .insert("v".into(), format!("0x{:x}", v).into());
            Ok(tx)
        }
    }
}

#[tokio::test]
async fn test_eth_get_transaction_by_hash() {
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

    let typed_tx = TypedTransaction::Eip1559(tx.clone());

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

    let result = eth_get_transaction_by_hash(_ctx.into(), params).await;

    let mut _tx = serde_json::to_value(&signed_tx).unwrap();
    let sig = signed_tx.as_eip1559().unwrap().signature();

    assert!(result.is_ok());

    let expected_tx_json = json!({
        "accessList": [{
            "address": "0x0000000000000000000000000000000000000000",
            "storageKeys": ["0x0000000000000000000000000000000000000000000000000000000000000000"]
        }],
        "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "blockNumber": format!("0x{:x}", included_tx.block_number),
        "chainId": format!("0x{:x}", tx.clone().chain_id),
        "from": format!("0x{}", signed_tx.recover_signer().unwrap().encode_hex_upper()),
        "gas": "0x0",
        "gasPrice": "0x0",
        "hash": format!("0x{}", tx_hash.encode_hex()),
        "input": "0x68656c6c6f",
        "maxFeePerGas": format!("0x{:x}", tx.clone().max_fee_per_gas),
        "maxPriorityFeePerGas": format!("0x{:x}", tx.clone().max_priority_fee_per_gas),
        "nonce": format!("0x{:x}", 0),
        "r": format!("0x{:x}", sig.clone().r()),
        "s": format!("0x{:x}", sig.clone().s()),
        "to": "0x0606060606060606060606060606060606060606",
        "transactionIndex": format!("0x{:x}", included_tx.transaction_index),
        "type": "0x2",
        "v": format!("0x{:x}", sig.clone().v().to_u64()),
        "value": "0x0",
        "yParity": format!("0x{:x}", sig.clone().v().to_u64()),
    });

    assert_eq!(result.unwrap(), expected_tx_json);
}
