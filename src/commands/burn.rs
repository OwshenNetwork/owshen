use std::path::PathBuf;

use ethers::{
    prelude::*,
    providers::{Http, Middleware, Provider},
    types::TransactionRequest,
};
use structopt::StructOpt;

use crate::{config::Wallet, helper::to_wei};

#[derive(StructOpt, Debug)]
pub struct BurnOpt {
    #[structopt(long)]
    amount: f64,
    #[structopt(long)]
    priv_src: String,
    #[structopt(long)]
    endpoint: String,
    #[structopt(long)]
    chain_id: u64,
}

pub async fn burn(_opt: BurnOpt, wallet_path: PathBuf) {
    let provider = Provider::<Http>::try_from(_opt.endpoint.clone());
    let provider = match provider {
        Ok(provider) => provider,
        Err(e) => {
            println!("Error: failed to create provider: {:?}", e);
            return;
        }
    };

    let wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();

    let mut wallet = wallet.unwrap_or_else(|| {
        panic!("Wallet is not initialized!");
    });

    let burn_address = wallet.derive_burn_addr();
    let amount_wei = to_wei(_opt.amount);

    let eth_account = _opt.priv_src.parse::<LocalWallet>().unwrap_or_else(|e| {
        panic!("Error: failed to parse private key: {:?}", e);
    });

    let tx = TransactionRequest::pay(burn_address.address, amount_wei)
        .from(eth_account.address())
        .chain_id(_opt.chain_id);

    let receipt = provider
        .send_transaction(tx, None)
        .await
        .unwrap_or_else(|e| {
            panic!("Error: failed to send transaction: {:?}", e);
        })
        .log_msg("Pending transfer")
        .await;
    match receipt {
        Ok(_) => {
            println!("Burnt {} DIVE to {:?}", _opt.amount, burn_address.address);
        }
        Err(e) => {
            println!("Error: failed to burn DIVE: {:?}", e);
            return;
        }
    };

    wallet.burnt_addresses.push(burn_address);
    wallet.save_wallet(wallet_path).unwrap_or_else(|e| {
        panic!("Error: {:?}", e);
    });
}
