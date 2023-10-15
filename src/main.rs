mod fp;
mod hash;
mod keys;
mod proof;
mod tree;

#[macro_use]
extern crate lazy_static;

use axum::{
    extract,
    response::{Html, Json},
    routing::get,
    Router,
};
use bindings::owshen::{Owshen, Point as OwshenPoint};
use tower_http::cors::CorsLayer;

use ethers::prelude::*;

use eyre::Result;
use keys::{EphemeralKey, PrivateKey, PublicKey};
use proof::prove;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;

use keys::Point;
use proof::Proof;
use structopt::StructOpt;
use tree::SparseMerkleTree;

// Initialize wallet, TODO: let secret be derived from a BIP-39 mnemonic code
#[derive(StructOpt, Debug)]
pub struct InitOpt {
    endpoint: String,
}

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
    Init(InitOpt),
    Info(InfoOpt),
    Deposit(DepositOpt),
    Withdraw(WithdrawOpt),
    Wallet(WalletOpt),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetInfoResponse {
    address: PublicKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetStealthRequest {
    address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetStealthResponse {
    address: PublicKey,
    ephemeral: EphemeralKey,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct GetWithdrawResponse {
    proof: Proof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Wallet {
    priv_key: PrivateKey,
    endpoint: String,
}

const PARAMS_FILE: &str = "contracts/circuits/coin_withdraw_0001.zkey";

async fn serve_wallet(pub_key: PublicKey) -> Result<()> {
    let info_addr = pub_key.clone();
    let app = Router::new()
        .route(
            "/withdraw",
            get(|| async {
                Json(GetWithdrawResponse {
                    proof: Default::default(),
                })
            }),
        )
        .route(
            "/stealth",
            get(
                |extract::Query(req): extract::Query<GetStealthRequest>| async move {
                    let pub_key = PublicKey::from_str(&req.address).unwrap();
                    let (ephemeral, address) = pub_key.derive(&mut rand::thread_rng());
                    Json(GetStealthResponse { address, ephemeral })
                },
            ),
        )
        .route(
            "/info",
            get(move || async move { Json(GetInfoResponse { address: info_addr }) }),
        )
        .layer(CorsLayer::permissive());

    const API_PORT: u16 = 8000;
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    let frontend = async {
        task::spawn_blocking(move || {
            let output = Command::new("npm")
                .arg("start")
                .current_dir("client")
                .spawn()
                .expect("failed to execute process");
        });
        Ok::<(), eyre::Error>(())
    };
    let backend = async {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
        Ok::<(), eyre::Error>(())
    };

    tokio::try_join!(backend, frontend)?;
    Ok(())
}

impl Into<OwshenPoint> for Point {
    fn into(self) -> OwshenPoint {
        OwshenPoint {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let wallet_path = home::home_dir().unwrap().join(".owshen-wallet.json");

    let wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Init(InitOpt { endpoint }) => {
            if wallet.is_none() {
                let wallet = Wallet {
                    priv_key: PrivateKey::generate(&mut rand::thread_rng()),
                    endpoint,
                };
                std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
            } else {
                println!("Wallet is already initialized!");
            }
        }
        OwshenCliOpt::Wallet(WalletOpt {}) => {
            if let Some(wallet) = &wallet {
                serve_wallet(wallet.priv_key.clone().into()).await?;
            } else {
                println!("Wallet is not initialized!");
            }
        }
        OwshenCliOpt::Info(InfoOpt {}) => {
            if let Some(wallet) = &wallet {
                println!(
                    "Owshen Address: {}",
                    PublicKey::from(wallet.priv_key.clone())
                );
            } else {
                println!("Wallet is not initialized!");
            }
        }
        OwshenCliOpt::Deposit(DepositOpt { to }) => {
            if let Some(wallet) = &wallet {
                // Transfer ETH to the Owshen contract and create a new commitment
                println!("Depositing a coin to Owshen address: {}", to);

                let port = 8545u16;
                let url = format!("http://localhost:{}", port).to_string();
                let provider = Provider::<Http>::try_from(url).unwrap();
                let provider = Arc::new(provider);

                let poseidon2_addr = deploy(
                    provider.clone(),
                    include_str!("assets/mimc7.abi"),
                    include_str!("assets/mimc7.evm"),
                )
                .await
                .address();

                let accounts = provider.get_accounts().await.unwrap();
                let from = accounts[0];

                let owshen = Owshen::deploy(provider.clone(), (poseidon2_addr))
                    .unwrap()
                    .legacy()
                    .from(from)
                    .send()
                    .await
                    .unwrap();

                let (ephkey, pubkey) =
                    PublicKey::from(wallet.priv_key.clone()).derive(&mut rand::thread_rng());

                owshen
                    .deposit(pubkey.point.into(), ephkey.point.into())
                    .legacy()
                    .from(from)
                    .value(U256::try_from("1000000000000000000").unwrap())
                    .call()
                    .await
                    .unwrap();

                let mut smt = SparseMerkleTree::new(32);
                smt.set(0, crate::hash::hash(pubkey.point.x, pubkey.point.y));
                let merkle_proof = smt.get(0);

                let stealthpriv = wallet.priv_key.derive(ephkey);
                let zkproof = prove(
                    PARAMS_FILE,
                    0,
                    stealthpriv.secret,
                    merkle_proof.proof.try_into().unwrap(),
                )?;

                let nullifier = stealthpriv.nullifier(0);

                owshen
                    .withdraw(
                        nullifier.into(),
                        bindings::owshen::Proof {
                            a: zkproof.a.into(),
                            b: zkproof.b.into(),
                            c: zkproof.c.into(),
                        },
                    )
                    .legacy()
                    .from(from)
                    .call()
                    .await
                    .unwrap();
            } else {
                println!("Wallet is not initialized!");
            }
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
use ethers::abi::Abi;

async fn deploy(
    client: Arc<Provider<Http>>,
    abi: &str,
    bytecode: &str,
) -> ContractInstance<Arc<Provider<Http>>, Provider<Http>> {
    let from = client.get_accounts().await.unwrap()[0];
    let abi = serde_json::from_str::<Abi>(abi).unwrap();
    let bytecode = Bytes::from_str(bytecode).unwrap();
    let factory = ContractFactory::new(abi, bytecode, client);
    let mut deployer = factory.deploy(()).unwrap().legacy();
    deployer.tx.set_from(from);
    let contract = deployer.send().await.unwrap();
    contract
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash;
    use bindings::coin_withdraw_verifier::CoinWithdrawVerifier;
    use ethers::abi::Abi;
    use ethers::utils::Ganache;
    use std::sync::Arc;

    use ethers::core::types::Bytes;
    use ethers::middleware::contract::ContractFactory;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_mimc7() {
        let port = 8545u16;
        let url = format!("http://localhost:{}", port).to_string();

        let _ganache = Ganache::new().port(port).spawn();
        let provider = Provider::<Http>::try_from(url).unwrap();
        let provider = Arc::new(provider);
        let accounts = provider.get_accounts().await.unwrap();
        let from = accounts[0];

        let abi = serde_json::from_str::<Abi>(include_str!("assets/mimc7.abi")).unwrap();
        let bytecode = Bytes::from_str(include_str!("assets/mimc7.evm")).unwrap();

        let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let client = std::sync::Arc::new(client);

        let factory = ContractFactory::new(abi, bytecode, client);

        let mut deployer = factory.deploy(()).unwrap().legacy();
        deployer.tx.set_from(from);

        let contract = deployer.send().await.unwrap();

        let func = contract
            .method::<_, U256>("MiMCSponge", (U256::from(3), U256::from(11)))
            .unwrap();

        //let gas = func.clone().estimate_gas().await.unwrap();
        //assert_eq!(gas, 40566.into());

        let hash = func.clone().call().await.unwrap();

        assert_eq!(
            hash,
            U256::from_str_radix(
                "0x2e25f67c1ce6bdf965097b228987b3a1fd2be8069e36c354cbaf0b5dcef2ff6e",
                16
            )
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_poseidon() {
        let port = 8545u16;
        let url = format!("http://localhost:{}", port).to_string();

        let _ganache = Ganache::new().port(port).spawn();
        let provider = Provider::<Http>::try_from(url).unwrap();
        let provider = Arc::new(provider);
        let accounts = provider.get_accounts().await.unwrap();
        let from = accounts[0];

        let abi = serde_json::from_str::<Abi>(include_str!("assets/poseidon2.abi")).unwrap();
        let bytecode = Bytes::from_str(include_str!("assets/poseidon2.evm")).unwrap();

        let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let client = std::sync::Arc::new(client);

        let factory = ContractFactory::new(abi, bytecode, client);

        let mut deployer = factory.deploy(()).unwrap().legacy();
        deployer.tx.set_from(from);

        let contract = deployer.send().await.unwrap();

        let func = contract
            .method_hash::<_, U256>([41, 165, 242, 246], ([U256::from(123), U256::from(234)],))
            .unwrap();

        let gas = func.clone().estimate_gas().await.unwrap();
        assert_eq!(gas, 50349.into());

        let hash = func.clone().call().await.unwrap();

        assert_eq!(
            hash,
            U256::from_str_radix(
                "0x0e331f99e024251a3a17152d7562d6257edc99595f9169b4e3b122d58a0e9d62",
                16
            )
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_deposit() {
        let priv_key = PrivateKey {
            secret: 1234.into(),
        };
        let pub_key: PublicKey = priv_key.clone().into();
        let timestamp = 123u32;

        let mut smt = SparseMerkleTree::new(32);
        smt.set(123, 4567.into());
        smt.set(
            2345,
            hash(
                hash(pub_key.point.x, pub_key.point.y),
                (timestamp as u64).into(),
            ),
        );
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

        let proof = prove(
            PARAMS_FILE,
            2345,
            1234.into(),
            val.proof.try_into().unwrap(),
        )
        .unwrap();

        let verifier = CoinWithdrawVerifier::deploy(provider.clone(), ())
            .unwrap()
            .legacy()
            .from(from)
            .send()
            .await
            .unwrap();

        let verified = verifier
            .verify_proof(
                proof.a,
                proof.b,
                proof.c,
                [smt.root().into(), priv_key.nullifier(2345).into()],
            )
            .legacy()
            .from(from)
            .call()
            .await
            .unwrap();

        assert!(verified);

        drop(ganache);
    }
}
