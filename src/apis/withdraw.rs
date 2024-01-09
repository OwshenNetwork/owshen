use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::{Point, PrivateKey, PublicKey};
use crate::proof::{prove, Proof};
use crate::Context;
use crate::GetWithdrawRequest;
use crate::GetWithdrawResponse;
use crate::PARAMS_FILE;

use axum::{extract::Query, response::Json};
use ethers::prelude::*;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;

pub async fn withdraw(
    Query(req): Query<GetWithdrawRequest>,
    context_withdraw: Arc<Mutex<Context>>,
    context_tree: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
) -> Result<Json<GetWithdrawResponse>, eyre::Report> {
    let index = req.index;
    let coins = context_withdraw.lock().await.coins.clone();
    let address = req.address;
    let merkle_root = context_tree.lock().await.tree.clone();
    // Find a coin with the specified index
    let filtered_coin = coins.iter().find(|coin| coin.index == index);
    match filtered_coin {
        Some(coin) => {
            println!("desss {:?}", req.desire_amount);
            let u32_index: u32 = index.low_u32();
            let u64_index: u64 = index.low_u64();
            // get merkle proof
            let merkle_proof = merkle_root.get(u64_index);

            let pub_key: PublicKey = PublicKey::from_str(&address)?;
            let (ephemeral, stealth_pub_key) = pub_key.derive_random(&mut rand::thread_rng());
            let stealth_priv: PrivateKey = priv_key.derive(ephemeral);
            let shared_secret: Fp = stealth_priv.shared_secret(ephemeral);

            let amount: U256 = coin.amount;
            let fp_amount = Fp::try_from(amount)?;
            let fp_new_amount = Fp::from_str(&req.desire_amount)?;
            let obfuscated_remaining_amount = fp_amount - fp_new_amount;
            println!("remaining amount {:?}", obfuscated_remaining_amount);

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

            let proof: std::result::Result<Proof, eyre::Error> = prove(
                PARAMS_FILE,
                u32_index,
                hint_token_address,
                amount,
                fp_new_amount.into(),
                obfuscated_remaining_amount.into(),
                PublicKey::null(),
                stealth_pub_key,
                coin.priv_key.secret,
                merkle_proof.proof.try_into().unwrap(),
            );
            match proof {
                Ok(proof) => Ok(Json(GetWithdrawResponse {
                    proof,
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
