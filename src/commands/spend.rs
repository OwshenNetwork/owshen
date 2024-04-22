use std::{path::PathBuf, str::FromStr, sync::Arc};

use bindings::dive_token::DiveToken;

use ethers::{
    core::k256::elliptic_curve::SecretKey,
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::{Signer, Wallet as wallet},
};

use structopt::StructOpt;

use crate::{
    config::Wallet,
    helper::{proof_to_groth16_proof, to_wei},
    proof::spend_prove,
};

#[derive(StructOpt, Debug)]
pub struct SpendOpt {
    #[structopt(long)]
    priv_src: String,
    #[structopt(long)]
    endpoint: String,
    #[structopt(long)]
    chain_id: u64,
    #[structopt(long)]
    token_address: String,
    #[structopt(long)]
    amount: f64,
    #[structopt(long)]
    spend_zkey_path: String,
    #[structopt(long)]
    spend_witness_path: String,
    #[structopt(long)]
    prover_path: String,
}

pub async fn spend(_opt: SpendOpt, wallet_path: PathBuf) {
    let provider = Provider::<Http>::try_from(_opt.endpoint.clone()).unwrap_or_else(|e| {
        panic!("Error: failed to create provider: {:?}", e);
    });
    let owshen_wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();

    let mut owshen_wallet = owshen_wallet.unwrap_or_else(|| {
        panic!("Wallet is not initialized!");
    });

    let private_key_bytes = hex::decode(&_opt.priv_src).expect("Invalid hex string for from");
    let private_key: SecretKey<_> =
        SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    let wallet = wallet::from(private_key).with_chain_id(_opt.chain_id);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let eth_account = _opt.priv_src.parse::<LocalWallet>().unwrap_or_else(|e| {
        panic!("Error: failed to parse private key: {:?}", e);
    });

    let token_h160_address = H160::from_str(&_opt.token_address).unwrap_or_else(|e| {
        panic!("Error: failed to parse token address: {:?}", e);
    });

    let contract = DiveToken::new(token_h160_address, client.clone());

    println!("Burnt Coins:");
    for (i, burnt_coin) in owshen_wallet.burnt_coins.iter().enumerate() {
        println!(
            "#{}: Amount: {:?}, Encrypted: {:?}",
            i, burnt_coin.amount, burnt_coin.encrypted
        );
    }
    println!("Enter the index of the burnt coin to spend: ");
    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    let idx = input.trim().parse::<usize>().unwrap_or_else(|e| {
        panic!("Error: failed to parse index: {:?}", e);
    });

    let burnt_coin = owshen_wallet.burnt_coins.get(idx).unwrap_or_else(|| {
        panic!("Burnt coin not found!");
    });

    let wei_withraw_amount = to_wei(_opt.amount);
    let remaining_coin = owshen_wallet.derive_burnt_coin(
        burnt_coin.amount - wei_withraw_amount,
        burnt_coin.encrypted.clone(),
    );
    println!("Remaining Coin: {:?}", remaining_coin.get_balance());

    let proof = spend_prove(
        burnt_coin.amount,
        burnt_coin.salt,
        wei_withraw_amount,
        remaining_coin.salt,
        _opt.spend_zkey_path,
        _opt.spend_witness_path,
        _opt.prover_path,
    );
    let proof = proof.unwrap_or_else(|e| {
        panic!("Error: failed to prove: {:?}", e);
    });

    println!("Proof: {:?}", proof);

    let function = contract.spend_coin(
        burnt_coin.get_balance(),
        remaining_coin.get_balance(),
        wei_withraw_amount,
        eth_account.address(),
        proof_to_groth16_proof(proof),
    );
    let pending_res = function.send().await;
    match pending_res {
        Ok(_) => {
            owshen_wallet.burnt_coins.remove(idx);
            owshen_wallet.burnt_coins.push(remaining_coin);
            owshen_wallet.save_wallet(wallet_path).unwrap_or_else(|e| {
                panic!("Error: failed to save wallet: {:?}", e);
            });
        }
        Err(e) => {
            println!("Error: failed to send spend transaction: {:?}", e);
        }
    }
}
