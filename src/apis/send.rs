use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::{Point, PrivateKey, PublicKey};
use crate::proof::{prove, Proof};
use crate::Context;
use crate::GetSendRequest;
use crate::GetSendResponse;

use axum::{extract::Query, response::Json};
use ethers::prelude::*;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;

pub async fn send(
    Query(req): Query<GetSendRequest>,
    context_send: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
    witness_gen_path: String,
    params_fils: String,
) -> Result<Json<GetSendResponse>, eyre::Report> {
    let index = req.index;
    let new_amount = req.new_amount;
    let receiver_address = req.receiver_address;
    let address = req.address;
    let coins = context_send.lock().await.coins.clone();
    let merkle_root = context_send.lock().await.tree.clone();
    // Find a coin with the specified index
    let filtered_coin = coins.iter().find(|coin| coin.index == index);

    match filtered_coin {
        Some(coin) => {
            let u32_index: u32 = index.low_u32();
            let u64_index: u64 = index.low_u64();
            // get merkle proof
            let merkle_proof = merkle_root.get(u64_index);

            let address_pub_key = PublicKey::from_str(&address)?;
            let (address_ephemeral, address_stealth_pub_key) =
                address_pub_key.derive_random(&mut rand::thread_rng());

            let receiver_address_pub_key = PublicKey::from_str(&receiver_address)?;
            let (receiver_address_ephemeral, receiver_address_stealth_pub_key) =
                receiver_address_pub_key.derive_random(&mut rand::thread_rng());

            let stealth_priv: PrivateKey = priv_key.derive(address_ephemeral);
            let sender_shared_secret: Fp = stealth_priv.shared_secret(address_ephemeral);
            let receiver_shared_secret: Fp = stealth_priv.shared_secret(receiver_address_ephemeral);

            let amount: U256 = coin.amount;
            let fp_amount = Fp::try_from(amount)?;
            let u256_new_amount = U256::from_dec_str(&new_amount)?;
            let fp_new_amount = Fp::try_from(u256_new_amount)?;
            let remaining_amount = fp_amount - fp_new_amount;
            let hint_token_address = h160_to_u256(coin.uint_token);

            let obfuscated_sender_remaining_amount_with_secret: U256 =
                (remaining_amount + sender_shared_secret).into();

            let obfuscated_receiver_remaining_amount_with_secret: U256 =
                (fp_new_amount + receiver_shared_secret).into();

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
            let proof: std::result::Result<Proof, eyre::Error> = prove(
                params_fils,
                u32_index,
                hint_token_address,
                amount,
                u256_new_amount,
                remaining_amount.into(),
                receiver_address_stealth_pub_key,
                address_stealth_pub_key,
                coin.priv_key.secret,
                merkle_proof.proof.try_into().unwrap(),
                witness_gen_path,
            );
            match proof {
                Ok(proof) => Ok(Json(GetSendResponse {
                    proof,
                    token: coin.uint_token,
                    amount,
                    nullifier: coin.nullifier,
                    obfuscated_receiver_amount: obfuscated_receiver_remaining_amount_with_secret,
                    obfuscated_sender_amount: obfuscated_sender_remaining_amount_with_secret,
                    receiver_commitment: u256_calc_send_commitment,
                    sender_commitment: u256_calc_sender_commitment,
                    sender_ephemeral: address_ephemeral.point,
                    receiver_ephemeral: receiver_address_ephemeral.point,
                })),
                Err(_e) => Err(eyre::Report::msg(
                    "Something wrong while creating proof for send",
                )),
            }
        }
        None => {
            log::warn!("No coin with index {} found", index);
            Ok(Json(GetSendResponse {
                proof: Proof::default(),
                token: H160::default(),
                amount: U256::default(),
                nullifier: U256::default(),
                obfuscated_receiver_amount: U256::default(),
                obfuscated_sender_amount: U256::default(),
                receiver_commitment: U256::default(),
                sender_commitment: U256::default(),
                sender_ephemeral: Point {
                    x: Fp::default(),
                    y: Fp::default(),
                },
                receiver_ephemeral: Point {
                    x: Fp::default(),
                    y: Fp::default(),
                },
            }))
        }
    }
}
