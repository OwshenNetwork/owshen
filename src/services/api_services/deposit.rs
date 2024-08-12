use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::{clone, sync::Arc};
use tokio::sync::Mutex;

use super::Context;
use crate::{
    blockchain::Blockchain,
    db::Value,
    services::{ContextKvStore, ContextSigner},
    types::{CustomTx, CustomTxMsg, Mint, Token, ERC20},
};

#[derive(Debug, Deserialize, Clone)]
pub struct DepositRequest {
    pub tx_hash: String,
    pub token: String,
    pub amount: U256,
    pub address: Address,
}

#[derive(Serialize, Clone)]
pub struct DepositResponse {
    pub owshen_tx_hash: Option<FixedBytes<32>>,
    pub success: bool,
}

pub async fn deposit_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    Json(payload): Json<DepositRequest>,
) -> Result<Json<DepositResponse>, anyhow::Error> {
    let mut _ctx = ctx.lock().await;

    let contract_address = _ctx.chain.config().owshen;
    let t = payload.clone().token;

    let token = match t.as_str() {
        "native" => Token::Native,
        "erc20" => {
            let addr = Address::from_slice(&hex::decode(t.trim_start_matches("0x"))?);
            let decimals = _ctx.chain.get_token_decimal(addr)?;
            let symbol = _ctx.chain.get_token_symbol(addr)?;
            Token::Erc20(ERC20 {
                address: addr,
                decimals,
                symbol,
            })
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid token type: {}", t));
        }
    };

    let amount = payload.amount;
    let address = payload.address;
    let tx_hash = payload.tx_hash;
    let tx_hash_bytes = hex::decode(tx_hash.trim_start_matches("0x"))?;
    let tx_hash_fixed_bytes = alloy::primitives::FixedBytes::from_slice(&tx_hash_bytes);

    let chain_id = _ctx.chain.config().chain_id;
    let to_addr = match chain_id {
        1 => {
            const PROVIDER_API_KEY: &str = "YOUR_INFURA_PROJECT_ID";
            let provider_url =
                format!("https://mainnet.infura.io/v3/{}", PROVIDER_API_KEY).parse()?;
            let provider = ProviderBuilder::new().on_http(provider_url);
            let res = match Provider::get_transaction_by_hash(&provider, tx_hash_fixed_bytes).await
            {
                Ok(Some(transaction)) => transaction,
                Ok(None) => {
                    return Err(anyhow::anyhow!("Transaction not found"));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to get transaction by hash: {:?}",
                        e
                    ));
                }
            };

            let to_address = match res.to {
                Some(address) => address,
                None => {
                    return Err(anyhow::anyhow!("Transaction does not have a 'to' address"));
                }
            };
            to_address
        }
        1387 => contract_address,
        _ => {
            return Err(anyhow::anyhow!("Unsupported chain_id: {}", chain_id));
        }
    };

    if to_addr == contract_address {
        let raw_tx_bytes = hex::decode(tx_hash.trim_start_matches("0x"))?;
        let hah: &[u8] = raw_tx_bytes.as_ref();
        let tx = CustomTx::create(
            &mut _ctx.signer.clone(),
            chain_id,
            CustomTxMsg::MintTx(Mint {
                tx_hash: hah.to_vec(),
                user_tx_hash: tx_hash.clone(),
                token,
                amount,
                address,
            }),
        )
        .await?;

        if _ctx
            .chain
            .db
            .get(crate::db::Key::DepositedTransaction(tx_hash.clone()))?
            .is_some()
        {
            return Err(anyhow::anyhow!("Transaction already exists"));
        }

        _ctx.chain.db.put(
            crate::db::Key::DepositedTransaction(tx_hash.clone()),
            Some(Value::DepositedTransaction(tx_hash)),
        )?;

        let queue = &mut _ctx.tx_queue.queue();

        if queue.iter().any(|t| match t.hash() {
            Ok(hash) => hash == tx.hash().unwrap_or_default(),
            Err(_) => false,
        }) {
            return Err(anyhow::anyhow!("Transaction already exists"));
        }

        if _ctx
            .chain
            .db
            .get(crate::db::Key::TransactionHash(tx.hash()?))?
            .is_some()
        {
            return Err(anyhow::anyhow!("Transaction already exists"));
        }

        return Ok(Json(DepositResponse {
            owshen_tx_hash: Some(tx.hash()?),
            success: true,
        }));
    } else {
        return Err(anyhow::anyhow!("Transaction is invalid!"));
    }
}
