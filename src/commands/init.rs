use std::path::PathBuf;

use bip39::Mnemonic;
use colored::Colorize;
use structopt::StructOpt;

use crate::{config::Wallet, keys::Entropy};

#[derive(StructOpt, Debug)]
pub struct InitOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    mnemonic: Option<Mnemonic>,
}

async fn initialize_wallet(mnemonic: Option<Mnemonic>) -> Wallet {
    let entropy = if let Some(m) = mnemonic {
        Entropy::from_mnemonic(m)
    } else {
        Entropy::generate(&mut rand::thread_rng())
    };

    let wallet = Wallet { entropy };

    println!(
        "{} {}",
        "Your 12-word mnemonic phrase is:".bright_green(),
        wallet.entropy.to_mnemonic().unwrap()
    );
    println!(
        "{}",
        "PLEASE KEEP YOUR MNEMONIC PHRASE IN A SAFE PLACE OR YOU WILL LOSE YOUR FUNDS!"
            .bold()
            .bright_red()
    );

    wallet
}

pub async fn init(opt: InitOpt, wallet_path: PathBuf) {
    let wallet_path = opt.db.unwrap_or(wallet_path.clone());
    let wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();
    if wallet.is_none() {
        let wallet = initialize_wallet(opt.mnemonic).await;
        std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
    } else {
        println!("Wallet is already initialized!");
    }
}
