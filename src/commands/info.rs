use std::path::PathBuf;

use ethers::prelude::*;
use structopt::StructOpt;

use crate::{
    config::Wallet,
    keys::{PrivateKey, PublicKey},
};

#[derive(StructOpt, Debug)]
pub struct InfoOpt {
    #[structopt(long)]
    pub private_key: Option<String>,
}

pub async fn info(_opt: InfoOpt, wallet_path: PathBuf) -> Result<(), eyre::Report> {
    let wallet: Wallet = serde_json::from_str(&std::fs::read_to_string(&wallet_path)?)?;
    println!(
        "Owshen Address: {}",
        PublicKey::from(PrivateKey::from(wallet.entropy.clone()))
    );

    println!("Burn Addresses");
    for (i, burn_address) in wallet.burnt_addresses.iter().enumerate() {
        if burn_address.used.unwrap_or(false) {
            continue;
        }
        println!(
            "#{}: Address: {:?}, Preimage: {:?}",
            i, burn_address.address, burn_address.preimage
        );
    }

    println!("Burnt Coins");
    for (i, burnt_coin) in wallet.burnt_coins.iter().enumerate() {
        println!(
            "#{}: Amount: {:?}, Encrypted: {:?}",
            i, burnt_coin.amount, burnt_coin.encrypted
        );
    }

    if let Some(private_key) = &_opt.private_key {
        let eth_account = private_key.parse::<LocalWallet>().unwrap_or_else(|e| {
            panic!("Error: failed to parse private key: {:?}", e);
        });
        println!("Ethereum Address: {:?}", eth_account.address());
    }
    Ok(())
}
