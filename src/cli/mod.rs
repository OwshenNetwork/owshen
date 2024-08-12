use std::path::PathBuf;

use alloy::signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner};
use anyhow::{Ok, Result};

mod node;

use crate::db::{DiskKvStore, RamKvStore};
use hex::FromHex;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct StartOpt {
    #[structopt(long, default_value = "3000")]
    api_port: u16,
    #[structopt(long, default_value = "8645")]
    rpc_port: u16,
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    private_key: Option<String>,
    #[structopt(long, default_value = "https://eth.llamarpc.com")]
    provider_address: reqwest::Url,
}

impl StartOpt {
    fn parse_signing_key(&self) -> Result<PrivateKeySigner> {
        let private_key: String = match &self.private_key {
            Some(pk) => pk.to_string(),
            None => std::env::var("PRIVATE_KEY")?,
        };

        let private_key_str = if private_key.starts_with("0x") {
            &private_key[2..]
        } else {
            &private_key
        };

        let key_bytes = <[u8; 32]>::from_hex(private_key_str)?;

        let signer = PrivateKeySigner::from_signing_key(SigningKey::from_slice(&key_bytes)?);

        Ok(signer)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Owshen", about = "Owshen node software!")]
enum Opt {
    Start(StartOpt),
    Debug,
}

pub async fn cli() -> Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Start(opt) => {
            let signing_key = opt.parse_signing_key()?;
            if let Some(db) = opt.db {
                node::run_node(
                    DiskKvStore::new(db, 128)?,
                    opt.api_port,
                    opt.rpc_port,
                    opt.provider_address,
                    signing_key,
                )
                .await?;
            } else {
                node::run_node(
                    RamKvStore::new(),
                    opt.api_port,
                    opt.rpc_port,
                    opt.provider_address,
                    signing_key,
                )
                .await?;
            }
        }
        Opt::Debug => {
            println!("Nothing to do!");
        }
    }

    Ok(())
}
