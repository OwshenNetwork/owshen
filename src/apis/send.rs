use axum::extract::Query;
use axum::response::Json;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::Point;
use crate::keys::PublicKey;
use crate::proof::prove;
use crate::proof::Proof;
use crate::Context;
use crate::GetSendRequest;
use crate::GetSendResponse;
use crate::PARAMS_FILE;

pub async fn send(
    Query(req): Query<GetSendRequest>,
    context_send: Arc<Mutex<Context>>,
    context_tree_send: Arc<Mutex<Context>>,
) -> Result<Json<GetSendResponse>, eyre::Report> {
    let index = req.index;
    let new_amount = req.new_amount;
    let receiver_address = req.receiver_address;
    let address = req.address;

    let coins = context_send.lock().unwrap().coins.clone();
    let merkle_root = context_tree_send.lock().unwrap().tree.clone();
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
                address_pub_key.derive(&mut rand::thread_rng());

            let receiver_address_pub_key = PublicKey::from_str(&receiver_address)?;
            let (receiver_address_ephemeral, receiver_address_stealth_pub_key) =
                receiver_address_pub_key.derive(&mut rand::thread_rng());

            let amount: U256 = coin.amount;
            let str_amount: String = U256::to_string(&amount);

            let str_amount_num: i64 = str_amount.parse()?;
            let new_amount_num: i64 = new_amount.parse()?;

            let send_amount = U256::from_str(&new_amount)?;

            let min = str_amount_num - new_amount_num;

            let remaining_amount = min.to_string();

            let obfuscated_remaining_amount = amount - new_amount_num;
            let hint_token_address = h160_to_u256(coin.uint_token);

            // calc commitment one -> its for receiver
            let calc_send_commitment = hash4([
                receiver_address_stealth_pub_key.point.x,
                receiver_address_stealth_pub_key.point.y,
                Fp::from_str(&new_amount)?,
                Fp::try_from(hint_token_address)?,
            ]);

            let u256_calc_send_commitment = calc_send_commitment.into();

            // calc commitment two -> its for sender
            let calc_sender_commitment: Fp = hash4([
                address_stealth_pub_key.point.x,
                address_stealth_pub_key.point.y,
                Fp::from_str(&remaining_amount)?,
                Fp::try_from(hint_token_address)?,
            ]);

            let u256_calc_sender_commitment = calc_sender_commitment.into();

            let proof: std::result::Result<Proof, eyre::Error> = prove(
                PARAMS_FILE,
                u32_index,
                hint_token_address,
                amount,
                new_amount_num.into(),
                obfuscated_remaining_amount,
                receiver_address_stealth_pub_key,
                address_stealth_pub_key,
                coin.priv_key.secret,
                merkle_proof.proof.try_into().unwrap(),
            );

            match proof {
                Ok(proof) => Ok(Json(GetSendResponse {
                    proof,
                    token: coin.uint_token,
                    amount,
                    nullifier: coin.nullifier,
                    obfuscated_receiver_amount: send_amount,
                    obfuscated_sender_amount: obfuscated_remaining_amount,
                    receiver_commitment: u256_calc_send_commitment,
                    sender_commitment: u256_calc_sender_commitment,
                    sender_ephemeral: address_ephemeral.point,
                    receiver_ephemeral: receiver_address_ephemeral.point,
                })),
                Err(e) => Err(eyre::Report::msg(
                    "Something wrong while creating proof for send",
                )),
            }
        }
        None => {
            println!("No coin with index {} found", index);
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
