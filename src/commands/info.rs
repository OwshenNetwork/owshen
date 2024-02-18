use std::path::PathBuf;

use structopt::StructOpt;

use crate::{
    config::Wallet,
    keys::{PrivateKey, PublicKey},
};

#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

pub async fn info(_opt: InfoOpt, wallet_path: PathBuf) {
    let wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();
    if let Some(wallet) = &wallet {
        println!(
            "Owshen Address: {}",
            PublicKey::from(PrivateKey::from(wallet.entropy.clone()))
        );
    } else {
        println!("Wallet is not initialized!");
    }
}
