mod fp;
mod hash;
mod keys;
mod proof;
mod tree;

use bindings::coin_withdraw_verifier::CoinWithdrawVerifier;
use ethers::prelude::*;
use ethers::utils::Ganache;
use eyre::Result;
use keys::{PrivateKey, PublicKey};
use proof::prove;
use std::sync::Arc;
use structopt::StructOpt;
use tree::SparseMerkleTree;

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

// Test Owshen on Ganache
#[derive(StructOpt, Debug)]
pub struct TestOpt {}

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Info(InfoOpt),
    Deposit(DepositOpt),
    Withdraw(WithdrawOpt),
    Test(TestOpt),
}

#[tokio::main]
async fn main() -> Result<()> {
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

    const PARAMS_FILE: &str = "contracts/circuits/coin_withdraw_0001.zkey";

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Info(InfoOpt {}) => {
            let sk = PrivateKey::generate(&mut rand::thread_rng());
            println!("Owshen Address: {}", PublicKey::from(sk.clone()));
        }
        OwshenCliOpt::Deposit(DepositOpt { to }) => {
            // Transfer ETH to the Owshen contract and create a new commitment
            println!("Depositing a coin to Owshen address: {}", to);
        }
        OwshenCliOpt::Withdraw(WithdrawOpt { to }) => {
            // Prove you own a certain coin in the Owshen contract and retrieve rewards in the given ETH address
            println!(
                "Proof: {:?}",
                prove(PARAMS_FILE, 2345, val.value, val.proof.try_into().unwrap())?
            );
            println!("Withdraw a coin to Ethereum address: {}", to);
        }
        OwshenCliOpt::Test(TestOpt {}) => {
            // Deploy contract locally on Ganache and debug
            let port = 8545u16;
            let url = format!("http://localhost:{}", port).to_string();

            let ganache = Ganache::new().port(port).spawn();

            let provider = Provider::<Http>::try_from(url)?;
            let provider = Arc::new(provider);

            let accounts = provider.get_accounts().await?;
            let from = accounts[0];

            let proof = prove(PARAMS_FILE, 2345, val.value, val.proof.try_into().unwrap())?;

            let verifier = CoinWithdrawVerifier::deploy(provider.clone(), ())?
                .legacy()
                .from(from)
                .send()
                .await?;

            let verified = verifier
                .verify_proof(
                    proof.a,
                    proof.b,
                    proof.c,
                    proof.public.clone().try_into().unwrap(),
                )
                .legacy()
                .from(from)
                .call()
                .await?;

            if verified {
                println!("Proof verified successfully!");
            }

            drop(ganache);
        }
    }

    Ok(())
}
