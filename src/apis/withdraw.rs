use crate::checkpointed_hashchain::CheckpointedHashchainProof;
use crate::commands::wallet::Mode;
use crate::config::Context;
use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::{Point, PrivateKey, PublicKey};
use crate::proof::{prove, ProveResult};

use axum::{extract::Query, response::Json};
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetWithdrawRequest {
    index: U256,
    pub owshen_address: String,
    pub address: String,
    pub desire_amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetWithdrawResponse {
    proof: ProveResult,
    pub checkpoint_head: U256,
    pub latest_values_commitment_head: U256,
    pub token: H160,
    pub amount: U256,
    pub obfuscated_remaining_amount: U256,
    pub nullifier: U256,
    pub commitment: U256,
    pub ephemeral: Point,
}

pub async fn withdraw<P: AsRef<Path>>(
    Query(req): Query<GetWithdrawRequest>,
    context_withdraw: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
    witness_gen_path: P,
    prover_path: P,
    params_file: Option<P>,
    mode: Mode,
) -> Result<Json<GetWithdrawResponse>, eyre::Report> {
    let index = req.index;
    let owshen_address = req.owshen_address;
    let address = req.address;
    let coins = context_withdraw.lock().await.coins.clone();
    // let merkle_root = context_withdraw.lock().await.tree.clone();
    let chc = context_withdraw.lock().await.chc.clone();
    // Find a coin with the specified index
    let filtered_coin = coins.iter().find(|coin| coin.index == index);
    if let Some(coin) = filtered_coin {
        let u32_index: u32 = index.low_u32();
        let u64_index: u64 = index.low_u64();

        let chc_proof = chc.get(u64_index);

        let pub_key: PublicKey = PublicKey::from_str(&owshen_address)?;
        let (_, ephemeral, stealth_pub_key) = pub_key.derive_random(&mut rand::thread_rng());
        let stealth_priv: PrivateKey = priv_key.derive(ephemeral);
        let shared_secret: Fp = stealth_priv.shared_secret(ephemeral);

        let amount: U256 = coin.amount;
        let fp_amount = Fp::try_from(amount)?;
        let fp_new_amount = Fp::from_str(&req.desire_amount)?;
        let obfuscated_remaining_amount = fp_amount - fp_new_amount;

        let obfuscated_remaining_amount_with_secret: U256 =
            (obfuscated_remaining_amount + shared_secret).into();

        let remaining_amount = obfuscated_remaining_amount;

        let hint_token_address: U256 = h160_to_u256(coin.uint_token);

        let calc_commitment: Fp = hash4([
            stealth_pub_key.point.x,
            stealth_pub_key.point.y,
            remaining_amount,
            Fp::try_from(hint_token_address)?,
        ]);

        let u256_calc_commitment: U256 = calc_commitment.into();

        let indices: Vec<u32> = vec![u32_index, 0];
        let amounts: Vec<U256> = vec![amount, U256::from(0)];
        let secrets: Vec<Fp> = vec![coin.priv_key.secret, Fp::default()];
        let proofs: Vec<CheckpointedHashchainProof> = vec![chc_proof.clone(), chc_proof.clone()];
        let new_amounts: Vec<U256> = vec![fp_new_amount.into(), obfuscated_remaining_amount.into()];

        let h160_address = H160::from_str(&address)?;
        let null_pub_key = PublicKey {
            point: Point {
                x: Fp::try_from(h160_to_u256(h160_address))?,
                y: 0.into(),
            },
        };

        let pks: Vec<PublicKey> = vec![null_pub_key, stealth_pub_key];

        let prove_result = prove(
            hint_token_address,
            indices,
            amounts,
            secrets,
            proofs,
            new_amounts,
            pks,
            params_file.ok_or(eyre::Report::msg("Parameter file is not set!"))?,
            witness_gen_path,
            prover_path,
            &mode.clone(),
        )?;

        let checkpoint_head: U256 = chc_proof.checkpoint_head.into();
        let latest_values_commitment_head: U256 = chc_proof.latest_values_commitment_head.into();
        let proof_data = if mode == Mode::Windows {
            match prove_result {
                ProveResult::JsonInput(json) => ProveResult::JsonInput(json),
                _ => return Err(eyre::Report::msg("Expected JSON input for Windows mode")),
            }
        } else {
            match prove_result {
                ProveResult::Proof(proof) => ProveResult::Proof(proof),
                _ => {
                    return Err(eyre::Report::msg(
                        "Expected proof object for non-Windows mode",
                    ))
                }
            }
        };

        Ok(Json(GetWithdrawResponse {
            proof: proof_data,
            checkpoint_head,
            latest_values_commitment_head,
            token: coin.uint_token,
            amount,
            obfuscated_remaining_amount: obfuscated_remaining_amount_with_secret,
            nullifier: coin.nullifier,
            commitment: u256_calc_commitment,
            ephemeral: ephemeral.point,
        }))
    } else {
        log::warn!("No coin with index {} found", index);
        Err(eyre::Report::msg("Coin not found!"))
    }
}
