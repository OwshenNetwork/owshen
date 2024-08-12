use alloy::primitives::{keccak256, Signature};
use alloy::primitives::{FixedBytes, U256};
use alloy::sol_types::SolValue;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::Context;
use crate::config::CHAIN_ID;
use crate::services::{ContextKvStore, ContextSigner};
use crate::types::{CustomTx, Token, WithdrawCalldata};
use crate::{blockchain::Blockchain, types::Burn};

#[derive(Deserialize, Debug)]
pub struct WithdrawRequest {
    pub rlp_burn: Vec<u8>,
    pub sig: Signature,
}

#[derive(Serialize)]
pub struct WithdrawResponse {
    pub id: FixedBytes<32>,
    pub success: bool,
}

pub async fn withdraw_handler<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, anyhow::Error> {
    let mut _ctx = ctx.lock().await;

    let mut burn: Burn = rlp::decode(&payload.rlp_burn)?;
    let from_address = payload.sig.recover_address_from_msg(&payload.rlp_burn)?;
    _ctx.chain.db.put(
        crate::db::Key::Balance(from_address, Token::Native),
        Some(crate::db::Value::U256(U256::from(123456789))),
    )?;

    let burn_id = burn.burn_id.clone();
    if _ctx
        .chain
        .db
        .get(crate::db::Key::BurnId(burn_id.clone()))?
        .is_some()
    {
        return Err(anyhow::anyhow!("Burn id already used!"));
    }

    let calldata = WithdrawCalldata::Eth {
        address: from_address,
    };

    burn.calldata = Some(calldata);

    let tx = CustomTx::create(
        &mut _ctx.signer,
        CHAIN_ID,
        crate::types::CustomTxMsg::BurnTx(burn),
    )
    .await?;

    let id = tx.hash()?;
    _ctx.tx_queue.enqueue(tx);
    _ctx.chain.db.put(
        crate::db::Key::BurnId(burn_id),
        Some(crate::db::Value::Void),
    )?;

    Ok(Json(WithdrawResponse { id, success: true }))
}
