use std::{str::FromStr, sync::Arc};

use bindings::dive_token::DiveToken;
use ethers::{
    core::k256::elliptic_curve::SecretKey,
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::{Signer, Wallet as wallet},
};
use structopt::StructOpt;

use crate::helper::to_wei;

#[derive(StructOpt, Debug)]
pub struct ParticipateOpt {
    #[structopt(long)]
    pub endpoint: String,
    #[structopt(long)]
    pub chain_id: u64,
    #[structopt(long)]
    pub token_address: String,
    #[structopt(long)]
    pub private_key: String,
    #[structopt(long)]
    pub amount_per_epoch: f64,
    #[structopt(long)]
    pub num_epochs: u64,
}

pub async fn participate(_opt: ParticipateOpt) {
    let provider = Provider::<Http>::try_from(_opt.endpoint.clone()).unwrap_or_else(|e| {
        panic!("Error: failed to create provider: {:?}", e);
    });
    let private_key_bytes = hex::decode(&_opt.private_key).expect("Invalid hex string for from");
    let private_key: SecretKey<_> =
        SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    let wallet = wallet::from(private_key).with_chain_id(_opt.chain_id);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    let token_h160_address = H160::from_str(&_opt.token_address).unwrap_or_else(|e| {
        panic!("Error: failed to parse token address: {:?}", e);
    });
    let contract = DiveToken::new(token_h160_address, client.clone());

    let wei_amount_per_epoch = to_wei(_opt.amount_per_epoch);
    let approx = contract
        .approximate(wei_amount_per_epoch, U256::from(_opt.num_epochs))
        .call()
        .await
        .unwrap_or_else(|e| {
            panic!("Error: failed to get approximate: {:?}", e);
        });
    println!("Approximate: {:?}", approx);

    let approve_function = contract.approve(
        token_h160_address,
        wei_amount_per_epoch * U256::from(_opt.num_epochs),
    );
    let pending_res = approve_function.send().await;
    match pending_res {
        Ok(pending_res) => {
            pending_res.await.unwrap_or_else(|e| {
                panic!("Error: failed to approve: {:?}", e);
            });
        }
        Err(e) => {
            panic!("Error: failed to send approve transaction: {:?}", e);
        }
    }

    let participate_function =
        contract.participate(wei_amount_per_epoch, U256::from(_opt.num_epochs));
    let pending_res = participate_function.send().await;
    match pending_res {
        Ok(pending_res) => {
            pending_res.await.unwrap_or_else(|e| {
                panic!("Error: failed to participate: {:?}", e);
            });
        }
        Err(e) => {
            panic!("Error: failed to send participate transaction: {:?}", e);
        }
    }
}
