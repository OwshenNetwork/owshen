use alloy::primitives::{Address, Signature, U256};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::Context;
use crate::{
    blockchain::Blockchain,
    services::{ContextKvStore, ContextSigner},
    types::network::Network,
    types::Token,
    types::{CustomTxMsg, OwshenTransaction},
};

#[derive(Deserialize)]
pub struct WithdrawalsRequest {
    pub address: Address,
}

#[derive(Serialize)]
pub struct WithdrawalDetail {
    pub block_number: usize,
    pub signature: Signature,
    pub network: String,
    pub token: String,
    pub amount: U256,
}

#[derive(Serialize)]
pub struct WithdrawalsResponse {
    pub withdrawals: Vec<WithdrawalDetail>,
}

pub async fn withdrawals_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    Json(payload): Json<WithdrawalsRequest>,
) -> Result<Json<WithdrawalsResponse>, anyhow::Error> {
    let ctx_guard = ctx.lock().await;
    let blockchain = &ctx_guard.chain;

    let withdrawals = blockchain.get_user_withdrawals(payload.address)?;

    let withdrawal_details: Vec<WithdrawalDetail> = withdrawals
        .into_iter()
        .filter_map(|included_tx| {
            let tx: Result<OwshenTransaction, anyhow::Error> = included_tx.tx.try_into();

            if let Ok(OwshenTransaction::Custom(custom_tx)) = tx {
                if let Ok(CustomTxMsg::BurnTx(burn_data)) = custom_tx.msg() {
                    let network = match burn_data.network {
                        Network::ETH => "ETH".to_string(),
                        Network::BSC => "BSC".to_string(),
                    };
                    let token = match burn_data.token {
                        Token::Native => "Native".to_string(),
                        Token::Erc20(address) => format!("ERC20: {:?}", address),
                    };
                    let amount = burn_data.amount;
                    let signature = custom_tx.sig.clone();

                    return Some(WithdrawalDetail {
                        block_number: included_tx.block_number,
                        signature,
                        network,
                        token,
                        amount,
                    });
                }
            }
            None
        })
        .collect();

    Ok(Json(WithdrawalsResponse {
        withdrawals: withdrawal_details,
    }))
}
