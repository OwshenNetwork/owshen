mod fp;
mod hash;
mod keys;
mod proof;
mod tree;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use bindings::coin_withdraw_verifier::CoinWithdrawVerifier;
use ethers::prelude::*;
use ethers::utils::Ganache;
use eyre::Result;
use keys::{PrivateKey, PublicKey};
use proof::prove;
use std::net::SocketAddr;
use std::sync::Arc;
use structopt::StructOpt;
use tree::SparseMerkleTree;

// Open web wallet interface
#[derive(StructOpt, Debug)]
pub struct WalletOpt {}

// Show wallet info
#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

// Deposit to Owshen address
#[derive(StructOpt, Debug)]
pub struct DepositOpt {
    #[structopt(long)]
    to: PublicKey,
}

// Withdraw to Ethereum address
#[derive(StructOpt, Debug)]
pub struct WithdrawOpt {
    #[structopt(long)]
    to: Address,
}

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Info(InfoOpt),
    Deposit(DepositOpt),
    Withdraw(WithdrawOpt),
    Wallet(WalletOpt),
}

const PARAMS_FILE: &str = "contracts/circuits/coin_withdraw_0001.zkey";

async fn root() -> Html<&'static str> {
    Html(include_str!("html/wallet.html"))
}

async fn serve_wallet() -> Result<()> {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Running wallet on: http://127.0.0.1:8000");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let private_key = PrivateKey::from_secret(1234.into());

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Wallet(WalletOpt {}) => {
            serve_wallet().await?;
        }
        OwshenCliOpt::Info(InfoOpt {}) => {
            println!("Owshen Address: {}", PublicKey::from(private_key.clone()));
        }
        OwshenCliOpt::Deposit(DepositOpt { to }) => {
            // Transfer ETH to the Owshen contract and create a new commitment
            println!("Depositing a coin to Owshen address: {}", to);
        }
        OwshenCliOpt::Withdraw(WithdrawOpt { to }) => {
            // Prove you own a certain coin in the Owshen contract and retrieve rewards in the given ETH address
            let mut smt = SparseMerkleTree::new(32);
            smt.set(123, 4567.into());
            smt.set(2345, 4567.into());
            smt.set(2346, 1234.into());
            smt.set(0, 11234.into());
            smt.set(12345678, 11234.into());
            let val = smt.get(2345);
            println!(
                "{:?}: {}",
                smt.root(),
                SparseMerkleTree::verify(smt.root(), 2345, &val)
            );
            println!(
                "Proof: {:?}",
                prove(PARAMS_FILE, 2345, val.value, val.proof.try_into().unwrap())?
            );
            println!("Withdraw a coin to Ethereum address: {}", to);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deposit() {
        let mut smt = SparseMerkleTree::new(32);
        smt.set(123, 4567.into());
        smt.set(2345, 4567.into());
        smt.set(2346, 1234.into());
        smt.set(0, 11234.into());
        smt.set(12345678, 11234.into());
        let val = smt.get(2345);

        let port = 8545u16;
        let url = format!("http://localhost:{}", port).to_string();

        let ganache = Ganache::new().port(port).spawn();

        let provider = Provider::<Http>::try_from(url).unwrap();
        let provider = Arc::new(provider);

        let accounts = provider.get_accounts().await.unwrap();
        let from = accounts[0];

        let proof = prove(PARAMS_FILE, 2345, val.value, val.proof.try_into().unwrap()).unwrap();

        let verifier = CoinWithdrawVerifier::deploy(provider.clone(), ())
            .unwrap()
            .legacy()
            .from(from)
            .send()
            .await
            .unwrap();

        let verified = verifier
            .verify_proof(proof.a, proof.b, proof.c, [smt.root().into()])
            .legacy()
            .from(from)
            .call()
            .await
            .unwrap();

        assert!(verified);

        drop(ganache);
    }
}
