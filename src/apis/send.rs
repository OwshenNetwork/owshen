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
pub struct GetSendRequest {
    pub index: U256,
    pub new_amount: String,
    pub receiver_address: String,
    pub address: String,
    pub memo: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSendResponse {
    proof: ProveResult,
    pub checkpoint_head: U256,
    pub latest_values_commitment_head: U256,
    pub nullifier: U256,
    pub receiver_commitment: U256,
    pub sender_commitment: U256,
    pub sender_ephemeral: Point,
    pub receiver_ephemeral: Point,
    pub obfuscated_receiver_amount: U256,
    pub obfuscated_sender_amount: U256,
    pub obfuscated_receiver_token_address: U256,
    pub obfuscated_sender_token_address: U256,
    pub memo: String,
}

pub async fn send<P: AsRef<Path>>(
    Query(req): Query<GetSendRequest>,
    context_send: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
    witness_gen_path: P,
    prover_path: P,
    params_file: Option<P>,
    mode: Mode,
) -> Result<Json<GetSendResponse>, eyre::Report> {
    let index = req.index;
    let new_amount = req.new_amount;
    let receiver_address = req.receiver_address;
    let address = req.address;
    let memo = req.memo;
    let coins = context_send.lock().await.coins.clone();
    let chc = context_send.lock().await.chc.clone();
    let filtered_coin = coins.iter().find(|coin| coin.index == index);

    if let Some(coin) = filtered_coin {
        let u32_index: u32 = index.low_u32();
        let u64_index: u64 = index.low_u64();

        let chc_proof = chc.get(u64_index);

        let address_pub_key = PublicKey::from_str(&address)?;
        let (_, address_ephemeral, address_stealth_pub_key) =
            address_pub_key.derive_random(&mut rand::thread_rng());

        let receiver_address_pub_key = PublicKey::from_str(&receiver_address)?;
        let (
            receiver_address_priv_ephemeral,
            receiver_address_pub_ephemeral,
            receiver_address_stealth_pub_key,
        ) = receiver_address_pub_key.derive_random(&mut rand::thread_rng());

        let stealth_priv: PrivateKey = priv_key.derive(address_ephemeral);
        let sender_shared_secret: Fp = stealth_priv.shared_secret(address_ephemeral);

        let receiver_shared_secret: Fp =
            receiver_address_priv_ephemeral.shared_secret(receiver_address_stealth_pub_key);

        let amount: U256 = coin.amount;
        let fp_amount = Fp::try_from(amount)?;
        let u256_new_amount = U256::from_dec_str(&new_amount)?;
        let fp_new_amount = Fp::try_from(u256_new_amount)?;
        let remaining_amount = fp_amount - fp_new_amount;

        let hint_token_address = h160_to_u256(coin.uint_token);
        let fp_hint_token_address = Fp::try_from(hint_token_address)?;

        let obfuscated_sender_remaining_amount_with_secret: U256 =
            (remaining_amount + sender_shared_secret).into();

        let obfuscated_receiver_remaining_amount_with_secret: U256 =
            (fp_new_amount + receiver_shared_secret).into();

        let obfuscated_sender_token_address: U256 =
            (fp_hint_token_address + sender_shared_secret).into();

        let obfuscated_receiver_token_address: U256 =
            (fp_hint_token_address + receiver_shared_secret).into();

        // calc commitment one -> its for receiver
        let calc_send_commitment = hash4([
            receiver_address_stealth_pub_key.point.x,
            receiver_address_stealth_pub_key.point.y,
            fp_new_amount,
            Fp::try_from(hint_token_address)?,
        ]);
        let u256_calc_send_commitment = calc_send_commitment.into();
        // calc commitment two -> its for sender
        let calc_sender_commitment: Fp = hash4([
            address_stealth_pub_key.point.x,
            address_stealth_pub_key.point.y,
            remaining_amount,
            Fp::try_from(hint_token_address)?,
        ]);
        let u256_calc_sender_commitment = calc_sender_commitment.into();

        let indices: Vec<u32> = vec![u32_index, 0];
        let amounts: Vec<U256> = vec![amount, U256::from(0)];
        let secrets: Vec<Fp> = vec![coin.priv_key.secret, Fp::default()];
        let proofs: Vec<CheckpointedHashchainProof> = vec![chc_proof.clone(), chc_proof.clone()];
        let new_amounts: Vec<U256> = vec![u256_new_amount, remaining_amount.into()];
        let pks: Vec<PublicKey> = vec![receiver_address_stealth_pub_key, address_stealth_pub_key];

        let proof = prove(
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
        Ok(Json(GetSendResponse {
            proof,
            checkpoint_head,
            latest_values_commitment_head,
            nullifier: coin.nullifier,
            obfuscated_receiver_amount: obfuscated_receiver_remaining_amount_with_secret,
            obfuscated_sender_amount: obfuscated_sender_remaining_amount_with_secret,
            obfuscated_receiver_token_address,
            obfuscated_sender_token_address,
            receiver_commitment: u256_calc_send_commitment,
            sender_commitment: u256_calc_sender_commitment,
            sender_ephemeral: address_ephemeral.point,
            receiver_ephemeral: receiver_address_pub_ephemeral.point,
            memo,
        }))
    } else {
        log::warn!("No coin with index {} found", index);
        Err(eyre::Report::msg("Coin not found!"))
    }
}
