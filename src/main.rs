mod apis;
mod fp;
mod hash;
mod keys;
mod poseidon;
mod proof;
mod tree;

use axum::{
    // body::Bytes,
    body::Body,
    extract::{self, Query},
    http::{Response, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, get_service},
    Router,
};
use bindings::owshen::{Owshen, Point as OwshenPoint};
use bindings::simple_erc_20::SimpleErc20;
use bip39::Mnemonic;
use colored::Colorize;
use ethers::prelude::*;
use eyre::Result;
use fp::Fp;
use hash::hash4;
use keys::Point;
use keys::{PrivateKey, PublicKey};
use proof::Proof;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{fs::read_to_string, process::Command};
use structopt::StructOpt;
use tokio::fs::File;
use tokio::task;
use tokio_util::codec::{BytesCodec, FramedRead};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeFile;
use tree::SparseMerkleTree;
use webbrowser;

#[macro_use]
extern crate lazy_static;

const GOERLI_ENDPOINT: &str = "https://ethereum-goerli.publicnode.com";
// Initialize wallet, TODO: let secret be derived from a BIP-39 mnemonic code
#[derive(StructOpt, Debug)]
pub struct InitOpt {
    #[structopt(long, default_value = GOERLI_ENDPOINT)]
    endpoint: String,
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    mnemonic: Option<Mnemonic>,
    #[structopt(long)]
    test: bool,
}

// Open web wallet interface
#[derive(StructOpt, Debug)]
pub struct WalletOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long, default_value = "8000")]
    port: u16,
    #[structopt(long, default_value = GOERLI_ENDPOINT)]
    endpoint: String,
    #[structopt(long, help = "Enable test mode")]
    test: bool,
    #[structopt(long)]
    config: Option<PathBuf>,
}
#[derive(StructOpt, Debug)]
pub struct ConfigOpt {
    #[structopt(long, default_value = GOERLI_ENDPOINT)]
    endpoint: String,
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    config: Option<PathBuf>,
    #[structopt(long)]
    test: bool,
}

// Show wallet info
#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Init(InitOpt),
    Info(InfoOpt),
    Wallet(WalletOpt),
    Config(ConfigOpt),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    address: PublicKey,
    erc20_abi: Abi,
    dive_contract: H160,
    owshen_contract: H160,
    owshen_abi: Abi,
    token_contracts: Vec<TokenInfo>,
    is_test: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GetCoinsResponse {
    coins: Vec<Coin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStealthRequest {
    address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStealthResponse {
    address: Point,
    ephemeral: Point,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetWithdrawRequest {
    index: U256,
    pub address: String,
    pub desire_amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetWithdrawResponse {
    proof: Proof,
    pub token: H160,
    pub amount: U256,
    pub obfuscated_remaining_amount: U256,
    pub nullifier: U256,
    pub commitment: U256,
    pub ephemeral: Point,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSendRequest {
    index: U256,
    pub new_amount: String,
    pub receiver_address: String,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSendResponse {
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TokenInfo {
    token_address: H160,
    symbol: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Wallet {
    entropy: Entropy,
    token_contracts: Vec<TokenInfo>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    endpoint: String,
    dive_contract_address: H160,
    owshen_contract_address: H160,
    owshen_contract_abi: Abi,
    erc20_abi: Abi,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: String::new(),
            endpoint: GOERLI_ENDPOINT.to_string(),
            dive_contract_address: H160::default(),
            owshen_contract_address: H160::default(),
            owshen_contract_abi: Abi::default(),
            erc20_abi: Abi::default(),
        }
    }
}

pub struct Context {
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

fn extract_token_amount(
    hint_token_address: U256,
    hint_amount: U256,
    shared_secret: Fp,
    commitment: Fp,
    stealth_pub: PublicKey,
) -> Result<Option<(Fp, Fp)>, eyre::Report> {
    let amount = Fp::try_from(hint_amount)? - shared_secret;
    let token_address = Fp::try_from(hint_token_address)? - shared_secret;

    let calc_commitment1 = hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        Fp::try_from(hint_amount)?,
        Fp::try_from(hint_token_address)?,
    ]);

    let calc_commitment2 = hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        amount,
        token_address,
    ]);

    let calc_commitment3 = hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        amount,
        Fp::try_from(hint_token_address)?,
    ]);

    let calc_commitment4 = hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        Fp::try_from(hint_amount)?,
        token_address,
    ]);

    if calc_commitment1 == commitment {
        let fp_hint_token_address = Fp::try_from(hint_token_address)?;
        let fp_hint_amount = Fp::try_from(hint_amount)?;
        return Ok(Some((fp_hint_token_address, fp_hint_amount)));
    } else if calc_commitment2 == commitment {
        return Ok(Some((token_address, amount)));
    } else if calc_commitment3 == commitment {
        let fp_hint_token_address = Fp::try_from(hint_token_address)?;
        return Ok(Some((fp_hint_token_address, amount)));
    } else if calc_commitment4 == commitment {
        let fp_hint_amount = Fp::try_from(hint_amount)?;
        return Ok(Some((token_address, fp_hint_amount)));
    }

    Ok(None)
}

fn handle_error<T: IntoResponse>(result: Result<T, eyre::Report>) -> impl IntoResponse {
    match result {
        Ok(a) => a.into_response(),
        Err(e) => {
            let error_message = format!("Internal server error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message)).into_response()
        }
    }
}

async fn serve_index(test: bool) -> impl IntoResponse {
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
    let index_path = if test {
        "client/build/index.html".to_string()
    } else {
        format!("{}/usr/share/owshen/client/index.html", app_dir_path)
    };

    println!("index path {}", index_path);
    match read_to_string(index_path) {
        Ok(contents) => Html(contents),
        Err(_) => Html("<h1>Error: Unable to read the index file</h1>".to_string()),
    }
}

async fn serve_file(file_path: PathBuf) -> impl IntoResponse {
    if let Ok(file) = File::open(file_path).await {
        let stream = FramedRead::new(file, BytesCodec::new());

        Response::new(Body::wrap_stream(stream))
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("File not found"))
            .unwrap()
    }
}

async fn serve_wallet(
    provider: Arc<Provider<Http>>,
    _port: u16,
    priv_key: PrivateKey,
    pub_key: PublicKey,
    owshen_contract: H160,
    dive_contract: H160,
    abi: Abi,
    erc20_abi: Abi,
    token_contracts: Vec<TokenInfo>,
    test: bool,
) -> Result<()> {
    let tree: SparseMerkleTree = SparseMerkleTree::new(16);
    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        tree,
    }));

    let info_addr: PublicKey = pub_key.clone();
    let coins_owshen_abi = abi.clone();
    let coins_owshen_address = owshen_contract.clone();
    let context_coin = context.clone();
    let context_tree = context.clone();
    let context_tree_send = context.clone();
    let context_withdraw = context.clone();
    let context_send = context.clone();
    let contract = Contract::new(coins_owshen_address, coins_owshen_abi, provider);
    let contract_clone = contract.clone();

    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
    let root_files_path = format!("{}/usr/share/owshen/client", app_dir_path);
    let static_files_path = format!("{}/usr/share/owshen/client/static", app_dir_path);

    let app = Router::new()
        .route("/", get(move || serve_index(test)))
        .route(
            "/static/*file",
            get(|params: extract::Path<String>| async move {
                let file_path = PathBuf::from(static_files_path).join(params.as_str());
                println!("file path {:?}", file_path);
                serve_file(file_path).await
            }),
        )
        .route(
            "/manifest.json",
            get_service(ServeFile::new(format!("{}/manifest.json", root_files_path))),
        )
        .route(
            "/asset-manifest.json",
            get_service(ServeFile::new(format!(
                "{}/asset-manifest.json",
                root_files_path
            ))),
        )
        .route(
            "/robots.txt",
            get_service(ServeFile::new(format!("{}/robots.txt", root_files_path))),
        )
        .route(
            "/coins",
            get(move || async move {
                handle_error(apis::coins(context_coin, contract_clone, priv_key).await)
            }),
        )
        .route(
            "/withdraw",
            get(
                move |extract::Query(req): extract::Query<GetWithdrawRequest>| async move {
                    handle_error(
                        apis::withdraw(Query(req), context_withdraw, context_tree, priv_key).await,
                    )
                },
            ),
        )
        .route(
            "/send",
            get(
                move |extract::Query(req): extract::Query<GetSendRequest>| async move {
                    handle_error(
                        apis::send(Query(req), context_send, context_tree_send, priv_key).await,
                    )
                },
            ),
        )
        .route(
            "/stealth",
            get(
                |extract::Query(req): extract::Query<GetStealthRequest>| async move {
                    handle_error(apis::stealth(Query(req)).await)
                },
            ),
        )
        .route(
            "/info",
            get(move || async move {
                handle_error(
                    apis::info(
                        info_addr,
                        dive_contract,
                        owshen_contract,
                        token_contracts,
                        abi,
                        erc20_abi,
                        test,
                    )
                    .await,
                )
            }),
        )
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    if test {
        let frontend = async {
            task::spawn_blocking(move || {
                let _output = Command::new("npm")
                    .arg("start")
                    .env(
                        "REACT_APP_OWSHEN_ENDPOINT",
                        format!("http://127.0.0.1:{}", 9000),
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
    } else {
        let server = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal());

        // Attempt to open the web browser
        if webbrowser::open(&format!("http://{}", addr)).is_err() {
            println!(
                "Failed to open web browser. Please navigate to http://{} manually",
                addr
            );
        }

        server.await.map_err(eyre::Report::new)?;
        Ok(())
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}

impl Into<OwshenPoint> for Point {
    fn into(self) -> OwshenPoint {
        OwshenPoint {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

async fn initialize_config(endpoint: String, name: String, is_test: bool) -> Config {
    if is_test {
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

        return Config {
            name,
            endpoint,
            owshen_contract_address: owshen.address(),
            owshen_contract_abi: owshen.abi().clone(),
            dive_contract_address: dive.address(),
            erc20_abi: dive.abi().clone(),
        };
    } else {
        return Config::default();
    }
}

async fn initialize_wallet(endpoint: String, mnemonic: Option<Mnemonic>, is_test: bool) -> Wallet {
    let mut token_contracts: Vec<TokenInfo> = Vec::new();
    let provider = Provider::<Http>::try_from(endpoint.clone()).unwrap();
    let provider = Arc::new(provider);
    let accounts = provider.get_accounts().await.unwrap();

    if is_test {
        let from = accounts[0];
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

        token_contracts.push(TokenInfo {
            token_address: test_token.address(),
            symbol: "WETH".to_string(),
        });
        token_contracts.push(TokenInfo {
            token_address: second_test_token.address(),
            symbol: "USDC".to_string(),
        });
    }

    let entropy = if let Some(m) = mnemonic {
        Entropy::from_mnemonic(m)
    } else {
        Entropy::generate(&mut rand::thread_rng())
    };

    let wallet = Wallet {
        entropy,
        token_contracts,
    };

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

#[tokio::main]
async fn main() -> Result<()> {
    let wallet_path = home::home_dir().unwrap().join(".owshen-wallet.json");
    let config_path = home::home_dir().unwrap().join(".config-wallet.json");

    println!(
        "{} {}",
        "Your wallet path:".bright_green(),
        wallet_path.to_string_lossy()
    );

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Init(InitOpt {
            endpoint,
            db,
            mnemonic,
            test,
        }) => {
            let wallet_path = db.unwrap_or(wallet_path.clone());
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();
            if wallet.is_none() {
                let wallet = initialize_wallet(endpoint, mnemonic, test).await;
                std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
            } else {
                println!("Wallet is already initialized!");
            }
        }
        OwshenCliOpt::Config(ConfigOpt {
            endpoint,
            name,
            config,
            test,
        }) => {
            let config_path = config.unwrap_or(config_path.clone());
            let config = std::fs::read_to_string(&config_path)
                .map(|s| {
                    let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
                    c
                })
                .ok();
            if config.is_none() {
                let config = initialize_config(endpoint, name, test).await;
                std::fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
            } else {
                println!("Config is already initialized!");
            }
        }
        OwshenCliOpt::Wallet(WalletOpt {
            db,
            port,
            endpoint,
            test,
            config,
        }) => {
            let wallet_path = db.unwrap_or(wallet_path.clone());
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();

            let config_path = config.unwrap_or(config_path.clone());
            let config = std::fs::read_to_string(&config_path)
                .map(|s| {
                    let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
                    c
                })
                .ok();

            if let Some(wallet) = &wallet {
                let config = config.clone().unwrap_or_default();
                let provider = Provider::<Http>::try_from(config.endpoint.clone()).unwrap();
                let provider = Arc::new(provider);
                let priv_key = wallet.entropy.clone().into();
                let pub_key = PublicKey::from(priv_key);

                serve_wallet(
                    provider,
                    port,
                    priv_key,
                    pub_key,
                    config.owshen_contract_address,
                    config.dive_contract_address,
                    config.owshen_contract_abi.clone(),
                    config.erc20_abi.clone(),
                    wallet.token_contracts.clone(),
                    test,
                )
                .await?;
            } else {
                if wallet.is_none() {
                    let wallet = initialize_wallet(endpoint, None, test).await;
                    std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
                } else {
                    println!("Wallet is already initialized!");
                }
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
                    PublicKey::from(PrivateKey::from(wallet.entropy.clone()))
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

use crate::keys::Entropy;

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
