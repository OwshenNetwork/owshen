use crate::config::Context;
use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::{Point, PrivateKey, PublicKey};
use crate::proof::{prove, Proof};

use axum::{extract::Query, response::Json};
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
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
    proof: Proof,
    pub root: U256,
    pub token: H160,
    pub amount: U256,
    pub obfuscated_remaining_amount: U256,
    pub nullifier: U256,
    pub commitment: U256,
    pub ephemeral: Point,
}

pub async fn withdraw(
    Query(req): Query<GetWithdrawRequest>,
    context_withdraw: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
    witness_gen_path: String,
    prover_path: String,
    params_file: String,
) -> Result<Json<GetWithdrawResponse>, eyre::Report> {
    let index = req.index;
    let owshen_address = req.owshen_address;
    let address = req.address;
    let coins = context_withdraw.lock().await.coins.clone();
    let merkle_root = context_withdraw.lock().await.tree.clone();
    // Find a coin with the specified index
    let filtered_coin = coins.iter().find(|coin| coin.index == index);
    match filtered_coin {
        Some(coin) => {
            let u32_index: u32 = index.low_u32();
            let u64_index: u64 = index.low_u64();
            // get merkle proof
            let merkle_proof = merkle_root.get(u64_index);

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
            let proofs: Vec<Vec<[Fp; 3]>> = vec![
                merkle_proof.proof.clone().try_into().unwrap(),
                merkle_proof.proof.clone().try_into().unwrap(),
            ];
            let new_amounts: Vec<U256> =
                vec![fp_new_amount.into(), obfuscated_remaining_amount.into()];

            let h160_address = H160::from_str(&address)?;
            let null_pub_key = PublicKey {
                point: Point {
                    x: Fp::try_from(h160_to_u256(h160_address))?,
                    y: 0.into(),
                },
            };

            let pks: Vec<PublicKey> = vec![null_pub_key, stealth_pub_key];

            let proof: std::result::Result<Proof, eyre::Error> = prove(
                hint_token_address,
                indices,
                amounts,
                secrets,
                proofs,
                new_amounts,
                pks,
                params_file,
                witness_gen_path,
                prover_path,
            );

            let root: U256 = merkle_root.root().into();
            match proof {
                Ok(proof) => Ok(Json(GetWithdrawResponse {
                    proof,
                    root: root,
                    token: coin.uint_token,
                    amount,
                    obfuscated_remaining_amount: obfuscated_remaining_amount_with_secret,
                    nullifier: coin.nullifier,
                    commitment: u256_calc_commitment,
                    ephemeral: ephemeral.point,
                })),
                Err(_e) => Err(eyre::Report::msg(
                    "Something wrong while creating proof for withdraw",
                )),
            }
        }
        None => {
            log::warn!("No coin with index {} found", index);
            Ok(Json(GetWithdrawResponse {
                proof: Proof::default(),
                root: U256::default(),
                token: H160::default(),
                amount: U256::default(),
                obfuscated_remaining_amount: U256::default(),
                nullifier: U256::default(),
                commitment: U256::default(),
                ephemeral: Point {
                    x: Fp::default(),
                    y: Fp::default(),
                },
            }))
        }
    }
}
