use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{FixedBytes, U256},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signer},
};
use axum::{
    body::{self, Body, HttpBody},
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
    config::{self, CHAIN_ID},
    db::{DiskKvStore, Key, KvStore, RamKvStore, Value},
    genesis::GENESIS,
    safe_signer::{self, SafeSigner},
    services::{api_services::api_routes, Context},
    types::{
        network::Network, BincodableOwshenTransaction, Burn, CustomTx, CustomTxMsg,
        IncludedTransaction, Mint, OwshenTransaction, Token,
    },
};

#[tokio::test]
async fn withdrawal_test() {
    let (ctx, app) = test_config().await;

    let value = U256::from(100);

    let signer = PrivateKeySigner::random();
    let tx = CustomTx::create(
        &mut signer.clone(),
        123,
        CustomTxMsg::BurnTx(Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: Network::ETH,
            token: Token::Native,
            amount: value,
            calldata: None,
        }),
    )
    .await
    .unwrap();

    let custom_tx = if let OwshenTransaction::Custom(custom_tx) = tx.clone() {
        custom_tx
    } else {
        panic!("Expected custom transaction");
    };
    let signature = custom_tx.sig.clone();
    let bincodable_tx = tx.clone().try_into().unwrap();

    let block_number = 4321;
    let included_tx = IncludedTransaction {
        tx: bincodable_tx,
        block_hash: FixedBytes::from([0u8; 32]),
        block_number: block_number,
        transaction_index: 1,
    };
    let key = Key::Transactions(signer.address());
    let mut transactions: Vec<IncludedTransaction> = Vec::new();
    transactions.push(included_tx.clone());

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .chain
            .db
            .put(key.clone(), Some(Value::Transactions(transactions)))
            .unwrap();
    }

    let address_str = format!("{}", signer.address());

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/Withdrawals")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "address": address_str.to_lowercase(),
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({
            "withdrawals": [{
                "amount": format!("{:#x}", value),
                "block_number": block_number,
                "network": format!("{:?}", Network::ETH),
                "signature": {
                    "r": format!("{:#x}", signature.r()),
                    "s": format!("{:#x}", signature.s()),
                    "yParity": format!("{:#x}", signature.v().to_u64()),
                },
                "token": format!("{:?}", Token::Native),
            }]
        })
    );
}

#[tokio::test]
async fn withdraw_test() {
    let (ctx, app) = test_config().await;

    let signer = PrivateKeySigner::random();
    let address = signer.address();

    let before_balance = ctx
        .lock()
        .await
        .chain
        .get_balance(Token::Native, address)
        .unwrap();
    assert_eq!(before_balance, U256::from(0));

    let base_value = U256::from(100);
    ctx.lock()
        .await
        .chain
        .db
        .put(
            Key::Balance(address, Token::Native),
            Some(Value::U256(base_value)),
        )
        .unwrap();
    let balance = ctx
        .lock()
        .await
        .chain
        .get_balance(Token::Native, address)
        .unwrap();
    assert_eq!(balance, base_value);

    let burn_obj = Burn {
        burn_id: FixedBytes::from([1u8; 32]),
        network: Network::ETH,
        token: Token::Native,
        amount: U256::from(100),
        calldata: None,
    };
    let burn_rlp = rlp::encode(&burn_obj);
    let sig = signer.sign_message(&burn_rlp).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/withdraw")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "rlp_burn": burn_rlp,
                        "sig": sig,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.get("success").unwrap(), &serde_json::Value::Bool(true));
}

#[tokio::test]
async fn deposit_test() {
    let (ctx, app) = test_config().await;

    let signer: PrivateKeySigner = PrivateKeySigner::random();
    let chain_id = ctx.lock().await.chain.config().chain_id;
    let vitalik = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .unwrap();
    let tx = TransactionRequest::default()
        .with_to(vitalik)
        .with_deploy_code(vec![0, 0, 0, 0])
        .with_nonce(1)
        .with_gas_limit(100)
        .with_max_fee_per_gas(100)
        .with_max_priority_fee_per_gas(100)
        .with_chain_id(chain_id)
        .with_value(U256::from(100_000_000_000_000_000u128));
    let wallet = EthereumWallet::new(signer.clone());
    let user_tx_hash = tx
        .clone()
        .build(&wallet)
        .await
        .unwrap()
        .tx_hash()
        .to_string();
    let tx = OwshenTransaction::Eth(tx.build(&wallet).await.unwrap());
    let hash_result = tx.hash();
    let fixed_bytes_tx_hash = hash_result.as_ref().unwrap();
    let vec8_tx_hash = fixed_bytes_tx_hash.to_vec();
    let token = Token::Native;
    let amount = U256::from(100_000_000_000_000_000u128);
    let address = signer.address();

    let txx = CustomTx::create(
        &mut ctx.lock().await.signer,
        chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: vec8_tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token,
            amount,
            address,
        }),
    )
    .await
    .unwrap();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/deposit")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({"tx_hash": user_tx_hash, "token": "native", "amount": amount, "address": address})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        json!({"owshen_tx_hash": format!("{:?}", txx.hash().unwrap()), "success": true})
    );

    let bincodable_tx = txx.clone().try_into().unwrap();

    let block_number = 4321;
    let included_tx = IncludedTransaction {
        tx: bincodable_tx,
        block_hash: FixedBytes::from([0u8; 32]),
        block_number,
        transaction_index: 1,
    };
    let mut transactions: Vec<IncludedTransaction> = Vec::new();
    transactions.push(included_tx.clone());

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .chain
            .db
            .put(
                Key::TransactionHash(txx.hash().unwrap()),
                Some(Value::Transaction(included_tx.clone())),
            )
            .unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/deposit")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({"tx_hash": user_tx_hash, "token": "native", "amount": amount, "address": address})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        json!({"message": "Transaction already exists", "error": true})
    );
}

async fn test_config() -> (
    Arc<tokio::sync::Mutex<Context<SafeSigner, RamKvStore>>>,
    Router,
) {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };

    let owner = SafeSigner::new(PrivateKeySigner::random());
    let ctx = Arc::new(Mutex::new(Context {
        signer: owner.clone(),
        exit: false,
        tx_queue: TransactionQueue::new(),
        chain: Owshenchain::new(conf, RamKvStore::new()),
    }));

    let app = api_routes(ctx.clone());

    (ctx, app)
}
