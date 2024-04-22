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
pub struct DiveOpt {
    #[structopt(long)]
    pub endpoint: String,
    #[structopt(long)]
    pub chain_id: u64,
    #[structopt(long)]
    pub token_address: String,
    #[structopt(long)]
    pub private_key: String,
}

pub async fn dive(_opt: DiveOpt) {
    let provider = Provider::<Http>::try_from(_opt.endpoint.clone());
    let provider = match provider {
        Ok(provider) => provider,
        Err(e) => {
            println!("Error: failed to create provider: {:?}", e);
            return;
        }
    };

    let private_key_bytes = hex::decode(&_opt.private_key).expect("Invalid hex string for from");
    let private_key: SecretKey<_> =
        SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    let wallet = wallet::from(private_key).with_chain_id(_opt.chain_id);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    let token_h160_address = H160::from_str(&_opt.token_address);
    let token_h160_address = match token_h160_address {
        Ok(token_h160_address) => token_h160_address,
        Err(e) => {
            println!("Error: failed to parse token address: {:?}", e);
            return;
        }
    };

    let contract = DiveToken::new(token_h160_address, client.clone());

    let eth_account = _opt.private_key.parse::<LocalWallet>();
    let eth_account = match eth_account {
        Ok(eth_account) => eth_account,
        Err(e) => {
            println!("Error: failed to parse private key: {:?}", e);
            return;
        }
    };

    println!("Dive Address: {:?}", eth_account.address());
    let balance = contract.balance_of(eth_account.address()).call().await;
    let balance = match balance {
        Ok(balance) => balance,
        Err(e) => {
            println!("Error: failed to get balance: {:?}", e);
            return;
        }
    };
    println!("Dive Balance: {:?}", balance);

    let burnt_balance = contract
        .get_burnt_balance(eth_account.address())
        .call()
        .await;
    let burnt_balance = match burnt_balance {
        Ok(burnt_balance) => burnt_balance,
        Err(e) => {
            println!("Error: failed to get burnt balance: {:?}", e);
            return;
        }
    };
    println!("Burnt Balance: {:?}", burnt_balance);

    let current_epoch = contract.current_epoch().call().await;
    let current_epoch = match current_epoch {
        Ok(current_epoch) => current_epoch,
        Err(e) => {
            println!("Error: failed to get current epoch: {:?}", e);
            return;
        }
    };
    println!("Current Epoch: {:?}", current_epoch);
}
