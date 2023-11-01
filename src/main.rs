mod fp;
mod hash;
mod keys;
mod proof;
mod tree;

#[macro_use]
extern crate lazy_static;

use axum::{extract, response::Json, routing::get, Router};
use bindings::dive_token::DiveToken;
use bindings::owshen::{Owshen, Point as OwshenPoint, SentFilter, WithdrawFilter};
use tower_http::cors::CorsLayer;

use ethers::prelude::*;

use eyre::Result;
use keys::{EphemeralKey, PrivateKey, PublicKey};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;
use tree::SparseMerkleTree;

use crate::fp::Fp;
use ff::PrimeField;
use keys::Point;
use proof::{prove, Proof};
use std::path::PathBuf;
use structopt::StructOpt;

// Initialize wallet, TODO: let secret be derived from a BIP-39 mnemonic code
#[derive(StructOpt, Debug)]
pub struct InitOpt {
    #[structopt(long, default_value = "http://127.0.0.1:8545")]
    endpoint: String,
    #[structopt(long)]
    db: Option<PathBuf>,
}

// Open web wallet interface
#[derive(StructOpt, Debug)]
pub struct WalletOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long, default_value = "8000")]
    port: u16,
}

// Show wallet info
#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Init(InitOpt),
    Info(InfoOpt),
    Wallet(WalletOpt),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetInfoResponse {
    address: PublicKey,
    dive_abi: Abi,
    dive_address: H160,
    contract_address: H160,
    contract_abi: Abi,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetStealthRequest {
    address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetStealthResponse {
    address: Point,
    ephemeral: Point,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetWithdrawRequest {
    index: U256,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coin {
    pub index: U256,
    pub token: H160,
    pub amount: U256,
    pub priv_key: PrivateKey,
    pub pub_key: PublicKey,
    pub nullifier: U256,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct GetWithdrawResponse {
    proof: Proof,
    pub token: H160,
    pub amount: U256,
    pub nullifier: U256,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct GetCoinsResponse {
    coins: Vec<Coin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Wallet {
    priv_key: PrivateKey,
    endpoint: String,
    dive_contract_address: H160,
    owshen_contract_address: H160,
    owshen_contract_abi: Abi,
    dive_contract_abi: Abi,
}

struct Context {
    coins: Vec<Coin>,
    tree: SparseMerkleTree,
}

const PARAMS_FILE: &str = "contracts/circuits/coin_withdraw_0001.zkey";

use std::sync::Mutex;

async fn serve_wallet(
    provider: Arc<Provider<Http>>,
    port: u16,
    priv_key: PrivateKey,
    pub_key: PublicKey,
    owshen_address: H160,
    dive_address: H160,
    abi: Abi,
    dive_abi: Abi,
) -> Result<()> {
    let info_addr = pub_key.clone();
    let coins_owshen_address = owshen_address.clone();
    let div_address = dive_address.clone();
    let coins_owshen_abi = abi.clone();
    let tree: SparseMerkleTree = SparseMerkleTree::new(32);
    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        tree,
    }));

    let context_coin = context.clone();

    let context_tree = context.clone();
    let context_withdraw = context.clone();
    let contract = Contract::new(coins_owshen_address, coins_owshen_abi, provider);
    let contract_clone = contract.clone();
    let app = Router::new()
        .route(
            "/coins",
            get(move || async move {
                let mut my_coins = Vec::new();
                let mut tree = SparseMerkleTree::new(32);
                for sent_event in contract_clone
                    .event::<SentFilter>()
                    .from_block(0)
                    .to_block(100)
                    .query()
                    .await
                    .unwrap()
                {
                    let ephemeral = Point {
                        x: Fp::from_str_vartime(&sent_event.ephemeral.x.to_string()).unwrap(),
                        y: Fp::from_str_vartime(&sent_event.ephemeral.y.to_string()).unwrap(),
                    };
                    let pubkey = Point {
                        x: Fp::from_str_vartime(&sent_event.pub_key.x.to_string()).unwrap(),
                        y: Fp::from_str_vartime(&sent_event.pub_key.y.to_string()).unwrap(),
                    };
                    let stealth_priv = priv_key.derive(EphemeralKey { point: ephemeral });
                    let stealth_pub: PublicKey = stealth_priv.clone().into();
                    let index: U256 = sent_event.index;
                    let u64_index: u64 = index.low_u64();
                    tree.set(
                        u64_index,
                        crate::hash::hash(stealth_pub.point.x, stealth_pub.point.y),
                    );

                    if stealth_pub.point == pubkey {
                        println!("ITS FOR US! :O");
                        my_coins.push(Coin {
                            index,
                            token: div_address,
                            amount: sent_event.amount,
                            nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                            priv_key: stealth_priv,
                            pub_key: stealth_pub,
                        });
                    }
                }

                for withdraw_event in contract_clone
                    .event::<WithdrawFilter>()
                    .from_block(0)
                    .to_block(100)
                    .query()
                    .await
                    .unwrap()
                {
                    for _coin in my_coins.clone() {
                        let coin_position = my_coins
                            .iter()
                            .position(|_coin| _coin.nullifier == withdraw_event.nullifier);
                        match coin_position {
                            Some(index) => {
                                my_coins.remove(index);
                            }
                            None => {}
                        }

                        println!(
                            "YOU SPEND YOUR DEPOSIT! nullifier:{:?}",
                            withdraw_event.nullifier
                        );
                    }
                }

                let mut ctx = context_coin.lock().unwrap();
                ctx.coins = my_coins.clone();
                ctx.tree = tree;

                Json(GetCoinsResponse {
                    coins: my_coins.clone(),
                })
            }),
        )
        .route(
            "/withdraw",
            get(
                move |extract::Query(req): extract::Query<GetWithdrawRequest>| async move {
                    let index = req.index;
                    let coins = context_withdraw.lock().unwrap().coins.clone();
                    let merkle_root = context_tree.lock().unwrap().tree.clone();
                    // Find a coin with the specified index
                    let filtered_coin = coins.iter().find(|coin| coin.index == index);
                    match filtered_coin {
                        Some(coin) => {
                            let u32_index: u32 = index.low_u32();
                            let u64_index: u64 = index.low_u64();
                            // get merkle proof
                            let merkle_proof = merkle_root.get(u64_index);
                            // make proof
                            let proof: std::result::Result<Proof, eyre::Error> = prove(
                                PARAMS_FILE,
                                u32_index,
                                coin.priv_key.secret,
                                merkle_proof.proof.try_into().unwrap(),
                            );

                            match proof {
                                Ok(proof) => Json(GetWithdrawResponse {
                                    proof,
                                    token: coin.token,
                                    amount: coin.amount,
                                    nullifier: coin.nullifier,
                                }),
                                Err(e) => {
                                    println!("Something wrong while creating proof{:?}", e);
                                    Json(GetWithdrawResponse {
                                        proof: Proof::default(),
                                        token: H160::default(),
                                        amount: U256::default(),
                                        nullifier: U256::default(),
                                    })
                                }
                            }
                        }
                        None => {
                            println!("No coin with index {} found", index);
                            Json(GetWithdrawResponse {
                                proof: Proof::default(),
                                token: H160::default(),
                                amount: U256::default(),
                                nullifier: U256::default(),
                            })
                        }
                    }
                },
            ),
        )
        .route(
            "/stealth",
            get(
                |extract::Query(req): extract::Query<GetStealthRequest>| async move {
                    let pub_key = PublicKey::from_str(&req.address).unwrap();
                    let (ephemeral, address) = pub_key.derive(&mut rand::thread_rng());
                    Json(GetStealthResponse {
                        address: address.point,
                        ephemeral: ephemeral.point,
                    })
                },
            ),
        )
        .route(
            "/info",
            get(move || async move {
                Json(GetInfoResponse {
                    address: info_addr,
                    dive_address: dive_address,
                    dive_abi: dive_abi,
                    contract_address: owshen_address,
                    contract_abi: abi,
                })
            }),
        )
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let frontend = async {
        task::spawn_blocking(move || {
            let _output = Command::new("npm")
                .arg("start")
                .env(
                    "REACT_APP_OWSHEN_ENDPOINT",
                    format!("http://127.0.0.1:{}", port),
                )
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

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Init(InitOpt { endpoint, db }) => {
            let wallet_path = db.unwrap_or(wallet_path.clone());
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();
            if wallet.is_none() {
                let provider = Provider::<Http>::try_from(endpoint.clone()).unwrap();
                let provider = Arc::new(provider);
                let _token_address: H160 =
                    H160::from_str("0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6").unwrap();

                println!("Deploying hash function...");
                let poseidon2_addr = deploy(
                    provider.clone(),
                    include_str!("assets/mimc7.abi"),
                    include_str!("assets/mimc7.evm"),
                )
                .await
                .address();

                let accounts = provider.get_accounts().await.unwrap();
                let from = accounts[0];

                println!("Deploying DIVE token...");
                let dive = DiveToken::deploy(
                    provider.clone(),
                    U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                )
                .unwrap()
                .legacy()
                .from(from)
                .send()
                .await
                .unwrap();

                println!("Deploying Owshen contract...");
                let owshen = Owshen::deploy(provider.clone(), poseidon2_addr)
                    .unwrap()
                    .legacy()
                    .from(from)
                    .send()
                    .await
                    .unwrap();
                let wallet = Wallet {
                    priv_key: PrivateKey::generate(&mut rand::thread_rng()),
                    endpoint,
                    owshen_contract_address: owshen.address(),
                    owshen_contract_abi: owshen.abi().clone(),
                    dive_contract_address: dive.address(),
                    dive_contract_abi: dive.abi().clone(),
                };
                std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
            } else {
                println!("Wallet is already initialized!");
            }
        }
        OwshenCliOpt::Wallet(WalletOpt { db, port }) => {
            let wallet_path = db.unwrap_or(wallet_path.clone());
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();

            if let Some(wallet) = &wallet {
                let provider = Provider::<Http>::try_from(wallet.endpoint.clone()).unwrap();
                let provider = Arc::new(provider);

                serve_wallet(
                    provider,
                    port,
                    wallet.priv_key.clone(),
                    wallet.priv_key.clone().into(),
                    wallet.owshen_contract_address,
                    wallet.dive_contract_address,
                    wallet.owshen_contract_abi.clone(),
                    wallet.dive_contract_abi.clone(),
                )
                .await?;
            } else {
                println!("Wallet is not initialized!");
            }
        }
        OwshenCliOpt::Info(InfoOpt {}) => {
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();
            if let Some(wallet) = &wallet {
                println!(
                    "Owshen Address: {}",
                    PublicKey::from(wallet.priv_key.clone())
                );
            } else {
                println!("Wallet is not initialized!");
            }
        }
    }

    Ok(())
}
use ethers::abi::Abi;
use ethers::types::H160;

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
    async fn test_deposit_withdraw() {
        let priv_key = PrivateKey {
            secret: 1234.into(),
        };
        let port = 8545u16;
        let url = format!("http://localhost:{}", port).to_string();
        //let _ganache = Ganache::new().port(port).spawn();

        let provider = Provider::<Http>::try_from(url).unwrap();
        let provider = Arc::new(provider);
        let token_address: H160 =
            H160::from_str("0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6").unwrap();

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

        let (ephkey, pubkey) = PublicKey::from(priv_key.clone()).derive(&mut rand::thread_rng());

        let mut smt = SparseMerkleTree::new(32);
        let root = owshen.root().legacy().from(from).call().await.unwrap();
        assert_eq!(root, smt.root().into());

        owshen
            .deposit(
                pubkey.point.into(),
                ephkey.point.into(),
                token_address,
                1000.into(),
                H160::from_str("0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1").unwrap(),
                owshen.address(),
            )
            .legacy()
            .from(from)
            .send()
            .await
            .unwrap();

        smt.set(0, crate::hash::hash(pubkey.point.x, pubkey.point.y));
        let merkle_proof = smt.get(0);

        let root = owshen.root().legacy().from(from).call().await.unwrap();
        assert_eq!(root, smt.root().into());

        let stealthpriv = priv_key.derive(ephkey);
        let zkproof = prove(
            PARAMS_FILE,
            0,
            stealthpriv.secret,
            merkle_proof.proof.try_into().unwrap(),
        )
        .unwrap();

        let nullifier = stealthpriv.nullifier(0);

        owshen
            .withdraw(
                nullifier.into(),
                bindings::owshen::Proof {
                    a: zkproof.a.into(),
                    b: zkproof.b.into(),
                    c: zkproof.c.into(),
                },
                token_address,
                1000.into(),
                H160::from_str("0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1").unwrap(),
            )
            .legacy()
            .from(from)
            .send()
            .await
            .unwrap();
        let nullifier = stealthpriv.nullifier(0);
    }
}
