mod apis;
mod fp;
mod hash;
mod keys;
mod poseidon;
mod proof;
mod tree;

#[macro_use]
extern crate lazy_static;

use axum::{extract, response::Json, routing::get, Router};
use bindings::owshen::{Owshen, Point as OwshenPoint, SentFilter, SpendFilter};
use bindings::simple_erc_20::SimpleErc20;
use std::net::SocketAddr;
use tokio::time::timeout;

use hash::hash4;
use tower_http::cors::CorsLayer;

use ethers::prelude::*;

use eyre::Result;
use keys::{EphemeralKey, PrivateKey, PublicKey};

use serde::{Deserialize, Serialize};
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
    erc20_abi: Abi,
    dive_contract: H160,
    owshen_contract: H160,
    owshen_abi: Abi,
    token_contracts: Vec<TokenInfo>,
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
    pub address: String,
    pub desire_amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetSendRequest {
    index: U256,
    pub new_amount: String,
    pub receiver_address: String,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetSendResponse {
    proof: Proof,
    pub token: H160,
    pub amount: U256,
    pub nullifier: U256,
    pub receiver_commitment: U256,
    pub sender_commitment: U256,
    pub sender_ephemeral: Point,
    pub receiver_ephemeral: Point,
    pub obfuscated_receiver_amount: U256,
    pub obfuscated_sender_amount: U256,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coin {
    pub index: U256,
    pub uint_token: H160,
    pub amount: U256,
    pub priv_key: PrivateKey,
    pub pub_key: PublicKey,
    pub nullifier: U256,
    pub commitment: U256,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Send {
    pub index: U256,
    pub token_address: H160,
    pub amount: U256,
    pub commitment: U256,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetWithdrawResponse {
    proof: Proof,
    pub token: H160,
    pub amount: U256,
    pub obfuscated_remaining_amount: U256,
    pub nullifier: U256,
    pub commitment: U256,
    pub ephemeral: Point,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct GetCoinsResponse {
    coins: Vec<Coin>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TokenInfo {
    token_address: H160,
    symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Wallet {
    priv_key: PrivateKey,
    endpoint: String,
    dive_contract_address: H160,
    owshen_contract_address: H160,
    owshen_contract_abi: Abi,
    erc20_abi: Abi,
    token_contracts: Vec<TokenInfo>,
}

struct Context {
    coins: Vec<Coin>,
    tree: SparseMerkleTree,
}

const PARAMS_FILE: &str = "contracts/circuits/coin_withdraw_0001.zkey";

fn u256_to_h160(u256: U256) -> H160 {
    let mut bytes: [u8; 32] = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    let address_bytes: &[u8] = &bytes[12..32]; // Taking the last 20 bytes for ethereum address
    H160::from_slice(address_bytes)
}

fn h160_to_u256(h160_val: H160) -> U256 {
    let mut bytes = [0u8; 32];
    bytes[12..32].copy_from_slice(h160_val.as_bytes());

    U256::from_big_endian(&bytes)
}

use std::sync::Mutex;

async fn serve_wallet(
    provider: Arc<Provider<Http>>,
    port: u16,
    priv_key: PrivateKey,
    pub_key: PublicKey,
    owshen_contract: H160,
    dive_contract: H160,
    abi: Abi,
    erc20_abi: Abi,
    token_contracts: Vec<TokenInfo>,
) -> Result<()> {
    let info_addr = pub_key.clone();
    let coins_owshen_address = owshen_contract.clone();
    let coins_owshen_abi = abi.clone();
    let tree: SparseMerkleTree = SparseMerkleTree::new(16);
    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        tree,
    }));

    let context_coin = context.clone();

    let context_tree = context.clone();
    let context_tree_send = context.clone();
    let context_withdraw = context.clone();
    let context_send = context.clone();
    let contract = Contract::new(coins_owshen_address, coins_owshen_abi, provider);
    let contract_clone = contract.clone();
    let app = Router::new()
        .route("/coins", get(move || async move {}))
        .route(
            "/withdraw",
            get(
                move |extract::Query(req): extract::Query<GetWithdrawRequest>| async move {
                    let index = req.index;
                    let coins = context_withdraw.lock().unwrap().coins.clone();
                    let address = req.address;
                    let merkle_root = context_tree.lock().unwrap().tree.clone();
                    // Find a coin with the specified index
                    let filtered_coin = coins.iter().find(|coin| coin.index == index);
                    match filtered_coin {
                        Some(coin) => {
                            let u32_index: u32 = index.low_u32();
                            let u64_index: u64 = index.low_u64();
                            // get merkle proof
                            let merkle_proof = merkle_root.get(u64_index);
                            let pub_key = PublicKey::from_str(&address).unwrap();
                            let (ephemeral, stealth_pub_key) =
                                pub_key.derive(&mut rand::thread_rng());

                            let amount: U256 = coin.amount;
                            let str_amount: String = U256::to_string(&amount);
                            let _desire_amount: U256 = U256::from_str(&req.desire_amount).unwrap();

                            let str_amount_num: i64 = str_amount.parse().unwrap();
                            let new_amount_num: i64 = req.desire_amount.parse().unwrap();

                            // let remaining_amount = str_amount_num - new_amount_num;s
                            let obfuscated_remaining_amount = amount - new_amount_num;

                            let min: i64 = str_amount_num - new_amount_num;
                            let remaining_amount = min.to_string();

                            let hint_token_address = h160_to_u256(coin.uint_token);

                            let calc_commitment = hash4([
                                stealth_pub_key.point.x,
                                stealth_pub_key.point.y,
                                Fp::from_str(&remaining_amount.to_string()).unwrap(),
                                Fp::from_str(&U256::to_string(&hint_token_address)).unwrap(),
                            ]);
                            let u256_calc_commitment = calc_commitment.into();

                            let proof: std::result::Result<Proof, eyre::Error> = prove(
                                PARAMS_FILE,
                                u32_index,
                                hint_token_address,
                                amount,
                                new_amount_num.into(),
                                obfuscated_remaining_amount,
                                PublicKey::null(),
                                stealth_pub_key,
                                coin.priv_key.secret,
                                merkle_proof.proof.try_into().unwrap(),
                            );
                            match proof {
                                Ok(proof) => Json(GetWithdrawResponse {
                                    proof,
                                    token: coin.uint_token,
                                    amount: coin.amount,
                                    obfuscated_remaining_amount,
                                    nullifier: coin.nullifier,
                                    commitment: u256_calc_commitment,
                                    ephemeral: ephemeral.point,
                                }),
                                Err(e) => {
                                    println!("Something wrong while creating proof{:?}", e);
                                    Json(GetWithdrawResponse {
                                        proof: Proof::default(),
                                        token: H160::default(),
                                        amount: U256::default(),
                                        obfuscated_remaining_amount: U256::default(),
                                        nullifier: U256::default(),
                                        commitment: U256::default(),
                                        ephemeral: ephemeral.point,
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
                                obfuscated_remaining_amount: U256::default(),
                                nullifier: U256::default(),
                                commitment: U256::default(),
                                ephemeral: Point {
                                    x: Fp::default(),
                                    y: Fp::default(),
                                },
                            })
                        }
                    }
                },
            ),
        )
        .route(
            "/send",
            get(
                move |extract::Query(req): extract::Query<GetSendRequest>| async move {
                    let index = req.index;
                    let new_amount = req.new_amount;
                    let receiver_address = req.receiver_address;
                    let address = req.address;

                    let coins = context_send.lock().unwrap().coins.clone();
                    let merkle_root = context_tree_send.lock().unwrap().tree.clone();
                    // Find a coin with the specified index
                    let filtered_coin = coins.iter().find(|coin| coin.index == index);

                    match filtered_coin {
                        Some(coin) => {
                            let u32_index: u32 = index.low_u32();
                            let u64_index: u64 = index.low_u64();
                            // get merkle proof
                            let merkle_proof = merkle_root.get(u64_index);

                            let address_pub_key = PublicKey::from_str(&address).unwrap();
                            let (address_ephemeral, address_stealth_pub_key) =
                                address_pub_key.derive(&mut rand::thread_rng());

                            let receiver_address_pub_key =
                                PublicKey::from_str(&receiver_address).unwrap();
                            let (receiver_address_ephemeral, receiver_address_stealth_pub_key) =
                                receiver_address_pub_key.derive(&mut rand::thread_rng());

                            let amount: U256 = coin.amount;
                            let str_amount: String = U256::to_string(&amount);

                            let str_amount_num: i64 = str_amount.parse().unwrap();
                            let new_amount_num: i64 = new_amount.parse().unwrap();

                            let send_amount = U256::from_str(&new_amount).unwrap();

                            let min = str_amount_num - new_amount_num;

                            let remaining_amount = min.to_string();

                            let obfuscated_remaining_amount = amount - new_amount_num;
                            let hint_token_address = h160_to_u256(coin.uint_token);

                            // calc commitment one -> its for receiver
                            let calc_send_commitment = hash4([
                                receiver_address_stealth_pub_key.point.x,
                                receiver_address_stealth_pub_key.point.y,
                                Fp::from_str(&new_amount).unwrap(),
                                Fp::from_str(&U256::to_string(&hint_token_address)).unwrap(),
                            ]);

                            let u256_calc_send_commitment = calc_send_commitment.into();

                            // calc commitment two -> its for sender
                            let calc_sender_commitment: Fp = hash4([
                                address_stealth_pub_key.point.x,
                                address_stealth_pub_key.point.y,
                                Fp::from_str(&remaining_amount).unwrap(),
                                Fp::from_str(&U256::to_string(&hint_token_address)).unwrap(),
                            ]);

                            let u256_calc_sender_commitment = calc_sender_commitment.into();

                            let proof: std::result::Result<Proof, eyre::Error> = prove(
                                PARAMS_FILE,
                                u32_index,
                                hint_token_address,
                                amount,
                                new_amount_num.into(),
                                obfuscated_remaining_amount,
                                receiver_address_stealth_pub_key,
                                address_stealth_pub_key,
                                coin.priv_key.secret,
                                merkle_proof.proof.try_into().unwrap(),
                            );

                            match proof {
                                Ok(proof) => Json(GetSendResponse {
                                    proof,
                                    token: coin.uint_token,
                                    amount,
                                    nullifier: coin.nullifier,
                                    obfuscated_receiver_amount: send_amount,
                                    obfuscated_sender_amount: obfuscated_remaining_amount,
                                    receiver_commitment: u256_calc_send_commitment,
                                    sender_commitment: u256_calc_sender_commitment,
                                    sender_ephemeral: address_ephemeral.point,
                                    receiver_ephemeral: receiver_address_ephemeral.point,
                                }),
                                Err(e) => {
                                    println!("Something wrong while creating proof{:?}", e);
                                    Json(GetSendResponse {
                                        proof: Proof::default(),
                                        token: H160::default(),
                                        amount: U256::default(),
                                        nullifier: U256::default(),
                                        obfuscated_receiver_amount: U256::default(),
                                        obfuscated_sender_amount: U256::default(),
                                        receiver_commitment: U256::default(),
                                        sender_commitment: U256::default(),
                                        sender_ephemeral: address_ephemeral.point,
                                        receiver_ephemeral: receiver_address_ephemeral.point,
                                    })
                                }
                            }
                        }
                        None => {
                            println!("No coin with index {} found", index);
                            Json(GetSendResponse {
                                proof: Proof::default(),
                                token: H160::default(),
                                amount: U256::default(),
                                nullifier: U256::default(),
                                obfuscated_receiver_amount: U256::default(),
                                obfuscated_sender_amount: U256::default(),
                                receiver_commitment: U256::default(),
                                sender_commitment: U256::default(),
                                sender_ephemeral: Point {
                                    x: Fp::default(),
                                    y: Fp::default(),
                                },
                                receiver_ephemeral: Point {
                                    x: Fp::default(),
                                    y: Fp::default(),
                                },
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
                    dive_contract,
                    erc20_abi,
                    owshen_contract,
                    owshen_abi: abi,
                    token_contracts,
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
                println!("Deploying hash function...");
                let poseidon4_addr = deploy(
                    provider.clone(),
                    include_str!("assets/poseidon4.abi"),
                    include_str!("assets/poseidon4.evm"),
                )
                .await
                .address();

                let accounts = provider.get_accounts().await.unwrap();
                let from = accounts[0];

                println!("Deploying DIVE token...");
                let dive = SimpleErc20::deploy(
                    provider.clone(),
                    (
                        U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                        String::from_str("dive_token").unwrap(),
                        String::from_str("DIVE").unwrap(),
                    ),
                )
                .unwrap()
                .legacy()
                .from(from)
                .send()
                .await
                .unwrap();
                println!("Deploying test tokens...");
                let test_token = SimpleErc20::deploy(
                    provider.clone(),
                    (
                        U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                        String::from_str("test_token").unwrap(),
                        String::from_str("TEST").unwrap(),
                    ),
                )
                .unwrap()
                .legacy()
                .from(from)
                .send()
                .await
                .unwrap();

                let second_test_token = SimpleErc20::deploy(
                    provider.clone(),
                    (
                        U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                        String::from_str("test_token").unwrap(),
                        String::from_str("TEST").unwrap(),
                    ),
                )
                .unwrap()
                .legacy()
                .from(from)
                .send()
                .await
                .unwrap();

                println!("Deploying Owshen contract...");
                let owshen = Owshen::deploy(provider.clone(), poseidon4_addr)
                    .unwrap()
                    .legacy()
                    .from(from)
                    .send()
                    .await
                    .unwrap();
                let mut token_contracts: Vec<TokenInfo> = Vec::new();

                token_contracts.push(TokenInfo {
                    token_address: test_token.address(),
                    symbol: "WETH".to_string(),
                });
                token_contracts.push(TokenInfo {
                    token_address: second_test_token.address(),
                    symbol: "USDC".to_string(),
                });

                let wallet = Wallet {
                    priv_key: PrivateKey::generate(&mut rand::thread_rng()),
                    endpoint,
                    owshen_contract_address: owshen.address(),
                    owshen_contract_abi: owshen.abi().clone(),
                    dive_contract_address: dive.address(),
                    erc20_abi: dive.abi().clone(),
                    token_contracts,
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
                    wallet.erc20_abi.clone(),
                    wallet.token_contracts.clone(),
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

    use ethers::abi::Abi;
    use ethers::utils::Ganache;

    use std::sync::Arc;

    use ethers::core::types::Bytes;
    use ethers::middleware::contract::ContractFactory;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_poseidon() {
        let port = 8545u16;
        let url = format!("http://localhost:{}", port).to_string();

        let _ganache = Ganache::new().port(port).spawn();
        let provider = Provider::<Http>::try_from(url).unwrap();
        let provider = Arc::new(provider);
        let accounts = provider.get_accounts().await.unwrap();
        let from = accounts[0];

        let abi = serde_json::from_str::<Abi>(include_str!("assets/poseidon4.abi")).unwrap();
        let bytecode = Bytes::from_str(include_str!("assets/poseidon4.evm")).unwrap();

        let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let client = std::sync::Arc::new(client);

        let factory = ContractFactory::new(abi, bytecode, client);

        let mut deployer = factory.deploy(()).unwrap().legacy();
        deployer.tx.set_from(from);

        let contract = deployer.send().await.unwrap();

        println!("{:?}", contract.methods);

        let func = contract
            .method_hash::<_, U256>(
                [36, 143, 102, 119],
                ([U256::from(0), U256::from(0), U256::from(0), U256::from(0)],),
            )
            .unwrap();

        let gas = func.clone().estimate_gas().await.unwrap();
        assert_eq!(gas, 91639.into());

        let hash = func.clone().call().await.unwrap();

        assert_eq!(
            hash,
            U256::from_str_radix(
                "0x0532fd436e19c70e51209694d9c215250937921b8b79060488c1206db73e9946",
                16
            )
            .unwrap()
        );
    }
}
