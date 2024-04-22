use std::path::PathBuf;

use structopt::StructOpt;

use crate::{
    config::Wallet,
    keys::{PrivateKey, PublicKey},
};

#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

pub async fn info(_opt: InfoOpt, wallet_path: PathBuf) -> Result<(), eyre::Report> {
    let wallet: Wallet = serde_json::from_str(&std::fs::read_to_string(&wallet_path)?)?;
    println!(
        "Owshen Address: {}",
        PublicKey::from(PrivateKey::from(wallet.entropy.clone()))
    );
    Ok(())
}
