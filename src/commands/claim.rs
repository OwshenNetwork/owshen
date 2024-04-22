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

#[derive(StructOpt, Debug)]
pub struct ClaimOpt {
    #[structopt(long)]
    pub endpoint: String,
    #[structopt(long)]
    pub chain_id: u64,
    #[structopt(long)]
    pub token_address: String,
    #[structopt(long)]
    pub private_key: String,
    #[structopt(long)]
    pub num_epochs: u64,
    #[structopt(long)]
    pub starting_epoch: u64,
}

pub async fn claim(_opt: ClaimOpt) {
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

    let function = contract.claim(U256::from(_opt.starting_epoch), U256::from(_opt.num_epochs));
    let pending_res = function.send().await;
    match pending_res {
        Ok(pending_res) => {
            pending_res.await.unwrap_or_else(|e| {
                panic!("Error: failed to claim: {:?}", e);
            });
        }
        Err(e) => {
            panic!("Error: failed to send claim transaction: {:?}", e);
        }
    }
}
