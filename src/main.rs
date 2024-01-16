mod apis;
mod fp;
mod genesis;
mod hash;
mod helper;
mod keys;
mod poseidon;
mod proof;
mod tree;

use axum::{
    body::Body,
    extract::{self, Query},
    http::{Response, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, get_service, post},
    Router,
};
use bindings::{
    owshen::{Owshen, Point as OwshenPoint},
    simple_erc_20::SimpleErc20,
};
use bip39::Mnemonic;
use colored::Colorize;
use ethers::{
    abi::Abi,
    core::k256::{elliptic_curve::SecretKey, Secp256k1},
    prelude::*,
    signers::Wallet as wallet,
    types::H160,
};
use eyre::Result;
use genesis::Genesis;
use helper::{h160_to_u256, u256_to_h160};
use hex::decode as hex_decode;
use keys::{Entropy, Point, PrivateKey, PublicKey};
use proof::Proof;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    {fs::read_to_string, process::Command},
};
use structopt::StructOpt;
use tokio::sync::Mutex;
use tokio::{fs::File, task};
use tokio_util::codec::{BytesCodec, FramedRead};
use tower_http::{cors::CorsLayer, services::ServeFile};
use tree::SparseMerkleTree;
use webbrowser;

#[macro_use]
extern crate lazy_static;

const GOERLI_ENDPOINT: &str = "https://ethereum-goerli.publicnode.com";
#[derive(StructOpt, Debug)]
pub struct InitOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    mnemonic: Option<Mnemonic>,
}
// Open web wallet interface
#[derive(StructOpt, Debug)]
pub struct WalletOpt {
    #[structopt(long)]
    db: Option<PathBuf>,
    #[structopt(long)]
    config: Option<PathBuf>,
    #[structopt(long, default_value = "8000")]
    port: u16,
    #[structopt(long, help = "Enable test mode")]
    test: bool,
}
#[derive(StructOpt, Debug)]
pub struct DeployOpt {
    #[structopt(long)]
    from: Option<String>,
    #[structopt(long, default_value = GOERLI_ENDPOINT)]
    endpoint: String,
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    config: Option<PathBuf>,
    #[structopt(long)]
    test: bool,
    #[structopt(long, default_value = "1337")]
    id: String,
    #[structopt(long)]
    deploy_dive: bool,
    #[structopt(long)]
    deploy_hash_function: bool,
    #[structopt(long)]
    deploy_owshen: bool,
    #[structopt(long)]
    genesis: bool,
}
// Show wallet info
#[derive(StructOpt, Debug)]
pub struct InfoOpt {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Empty {
    ok: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    address: PublicKey,
    erc20_abi: Abi,
    dive_contract: H160,
    owshen_contract: H160,
    owshen_abi: Abi,
    token_contracts: NetworkManager,
    is_test: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GetCoinsResponse {
    coins: Vec<Coin>,
    syncing: Option<f32>,
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
pub struct SetNetworkRequest {
    pub chain_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetNetworkResponse {
    pub success: Empty,
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
pub struct NetworkManager {
    networks: HashMap<String, Vec<TokenInfo>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Wallet {
    entropy: Entropy,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    name: String,
    endpoint: String,
    dive_contract_address: H160,
    owshen_contract_address: H160,
    owshen_contract_abi: Abi,
    erc20_abi: Abi,
    token_contracts: NetworkManager,
    poseidon_contract_address: H160,
}

impl NetworkManager {
    pub fn new() -> NetworkManager {
        let mut networks: HashMap<String, Vec<TokenInfo>> = HashMap::new();

        networks.insert(
            "ethereum_goerli".to_string(),
            vec![TokenInfo {
                token_address: H160::from_str("0xdD69DB25F6D620A7baD3023c5d32761D353D3De9")
                    .unwrap(),
                symbol: "WETH".to_string(),
            }],
        );

        NetworkManager { networks }
    }

    pub fn set(&mut self, data: HashMap<String, Vec<TokenInfo>>, expand: bool) {
        if expand {
            self.networks.extend(data);
        } else {
            self.networks = data;
        }
    }

    pub fn add_network(&mut self, network: String, token_info: Vec<TokenInfo>) {
        self.networks.insert(network, token_info);
    }

    pub fn get(&self, network: &str) -> Option<&Vec<TokenInfo>> {
        self.networks.get(network)
    }

    pub fn has(&self, network: &str, symbol: &str) -> bool {
        self.get(network).map_or(false, |tokens| {
            tokens.iter().any(|token_info| token_info.symbol == symbol)
        })
    }
}

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Init(InitOpt),
    Info(InfoOpt),
    Wallet(WalletOpt),
    Deploy(DeployOpt),
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
            token_contracts: NetworkManager::new(),
            poseidon_contract_address: H160::default(),
        }
    }
}
#[derive(Clone)]
pub struct Network {
    pub provider: Arc<Provider<Http>>,
    pub config: Config,
}
pub struct Context {
    coins: Vec<Coin>,
    tree: SparseMerkleTree,
    network: Option<Network>,
    genesis: Genesis,
    syncing: Arc<std::sync::Mutex<Option<f32>>>,
    syncing_task: Option<
        tokio::task::JoinHandle<
            std::result::Result<(tree::SparseMerkleTree, Vec<Coin>), eyre::Report>,
        >,
    >,
}

fn handle_error<T: IntoResponse>(result: Result<T, eyre::Report>) -> impl IntoResponse {
    match result {
        Ok(a) => a.into_response(),
        Err(e) => {
            log::error!("{}", e);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletCache {
    coins: Vec<Coin>,
    tree: SparseMerkleTree,
    height: u64,
}

async fn serve_wallet(
    _port: u16,
    priv_key: PrivateKey,
    pub_key: PublicKey,
    token_contracts: NetworkManager,
    test: bool,
    config: Config,
) -> Result<()> {
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());

    let params_file: String = if test {
        "contracts/circuits/coin_withdraw_0001.zkey".to_string()
    } else {
        format!("{}/usr/bin/coin_withdraw_0001.zkey", app_dir_path)
    };
    let send_params_file = params_file.clone();
    let genesis_path = if test {
        "owshen-genesis.dat".to_string()
    } else {
        format!(
            "{}/usr/share/genesis/{}-owshen-genesis.dat",
            app_dir_path, config.name
        )
    };
    let witness_gen_path = if test {
        "contracts/circuits/coin_withdraw_cpp/coin_withdraw".into()
    } else {
        format!("{}/usr/bin/coin_withdraw", app_dir_path).to_string()
    };

    let send_witness_gen_path = witness_gen_path.clone();
    let genesis: Option<Genesis> = if let Ok(f) = std::fs::read(genesis_path) {
        bincode::deserialize(&f).ok()
    } else {
        None
    };

    let context = Arc::new(Mutex::new(Context {
        coins: vec![],
        genesis: genesis.unwrap(),
        tree: SparseMerkleTree::new(16),
        network: None,
        syncing: Arc::new(std::sync::Mutex::new(None)),
        syncing_task: None,
    }));

    let info_addr: PublicKey = pub_key.clone();
    let context_coin = context.clone();
    let context_withdraw = context.clone();
    let context_send = context.clone();
    let context_info = context.clone();
    let contest_set_network = context.clone();
    let root_files_path = format!("{}/usr/share/owshen/client", app_dir_path);
    let static_files_path = format!("{}/usr/share/owshen/client/static", app_dir_path);

    let app = Router::new()
        .route("/", get(move || serve_index(test)))
        .route(
            "/static/*file",
            get(|params: extract::Path<String>| async move {
                let file_path = PathBuf::from(static_files_path).join(params.as_str());
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
            get(move || async move { handle_error(apis::coins(context_coin, priv_key).await) }),
        )
        .route(
            "/withdraw",
            get(
                move |extract::Query(req): extract::Query<GetWithdrawRequest>| async move {
                    handle_error(
                        apis::withdraw(
                            Query(req),
                            context_withdraw,
                            priv_key,
                            witness_gen_path,
                            params_file,
                        )
                        .await,
                    )
                },
            ),
        )
        .route(
            "/send",
            get(
                move |extract::Query(req): extract::Query<GetSendRequest>| async move {
                    handle_error(
                        apis::send(
                            Query(req),
                            context_send,
                            priv_key,
                            send_witness_gen_path,
                            send_params_file,
                        )
                        .await,
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
                handle_error(apis::info(info_addr, context_info, token_contracts, test).await)
            }),
        )
        .route(
            "/set-network",
            post(
                move |extract::Query(req): extract::Query<SetNetworkRequest>| async move {
                    handle_error(apis::set_network(Query(req), contest_set_network, test).await)
                },
            ),
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

async fn initialize_config(
    endpoint: String,
    from: String,
    name: String,
    is_test: bool,
    chain_id: String,
    config: Option<Config>,
    deploy_dive: bool,
    deploy_hash_function: bool,
    deploy_owshen: bool,
    genesis_feed: bool,
) -> Config {
    let mut network_manager = NetworkManager::new();
    let provider = Provider::<Http>::try_from(endpoint.clone()).unwrap();
    let provider = Arc::new(provider);
    let private_key_bytes = hex_decode(&from).expect("Invalid hex string for from");
    let private_key: SecretKey<_> =
        SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    let chain_id: u64 = chain_id.parse().expect("Invalid chain ID");
    let wallet = wallet::from(private_key.clone()).with_chain_id(chain_id);

    let from_address = if is_test {
        let accounts = provider.get_accounts().await.unwrap();
        accounts[0]
    } else {
        wallet.address()
    };

    let poseidon4_addr = if deploy_hash_function {
        log::info!("Deploying hash function...");
        deploy(
            provider.clone(),
            include_str!("assets/poseidon4.abi"),
            include_str!("assets/poseidon4.evm"),
            private_key.clone(),
            from_address,
            is_test,
            chain_id,
        )
        .await
    } else {
        if let Some(c) = &config {
            c.poseidon_contract_address
        } else {
            panic!("No config file!");
        }
    };
    println!("posidon address {:?}", poseidon4_addr);

    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let nonce = provider
        .get_transaction_count(from_address, None)
        .await
        .unwrap();

    println!("correct from address {:?}", from_address);
    let dive_contract_address = if deploy_dive {
        log::info!("Deploying DIVE token...");
        SimpleErc20::deploy(
            client.clone(),
            (
                U256::from_str_radix("89900000000000000000000", 10).unwrap(),
                String::from("dive_token"),
                String::from("DIVE"),
            ),
        )
        .unwrap()
        .legacy()
        .from(from_address)
        .nonce(nonce)
        .send()
        .await
        .unwrap()
        .address()
    } else {
        if let Some(c) = &config {
            c.dive_contract_address
        } else {
            panic!("No config file!");
        }
    };
    println!("dive address {:?}", dive_contract_address);

    let erc20_abi = serde_json::from_str::<Abi>(include_str!("assets/erc20.abi")).unwrap();
    let dive_contract = Contract::new(dive_contract_address, erc20_abi, client.clone());

    let genesis = if genesis_feed {
        log::info!("Filling the genesis tree... (This might take some time)");
        let genesis = genesis::fill_genesis(16, dive_contract_address);
        std::fs::write("owshen-genesis.dat", bincode::serialize(&genesis).unwrap()).unwrap();
        Some(genesis)
    } else {
        log::info!("Loading existing genesis tree...");
        if let Ok(f) = std::fs::read("owshen-genesis.dat") {
            bincode::deserialize(&f).ok()
        } else {
            log::warn!("No existing genesis data found. Proceeding without it.");
            None
        }
    };

    let genesis = match genesis {
        Some(ref g) => g,
        None => panic!("Genesis data is required but not available"),
    };

    let new_nonce = provider
        .get_transaction_count(from_address, None)
        .await
        .unwrap();

    let (owshen_contract_address, owshen_contract_abi) = if deploy_owshen {
        log::info!("Deploying Owshen contract...");
        let o = Owshen::deploy(
            client,
            (
                poseidon4_addr,
                Into::<U256>::into(genesis.smt.genesis_root()),
            ),
        )
        .unwrap()
        .legacy()
        .from(from_address)
        .nonce(new_nonce)
        .send()
        .await
        .unwrap();

        log::info!("Feeding DIVEs to the Owshen contract...");
        dive_contract
            .method::<_, bool>("transfer", (o.address(), Into::<U256>::into(genesis.total)))
            .unwrap()
            .legacy()
            .from(from_address)
            .send()
            .await
            .unwrap();

        (o.address(), o.abi().clone())
    } else {
        if let Some(c) = &config {
            (c.owshen_contract_address, c.owshen_contract_abi.clone())
        } else {
            panic!("No config file!");
        }
    };

    if is_test {
        let provider_url = "http://127.0.0.1:8545";
        let provider = Arc::new(Provider::<Http>::try_from(provider_url).unwrap());
        let accounts = provider.get_accounts().await.unwrap();
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

        let token_info1 = TokenInfo {
            token_address: test_token.address(),
            symbol: "WETH".to_string(),
        };

        let token_info2 = TokenInfo {
            token_address: second_test_token.address(),
            symbol: "USDC".to_string(),
        };

        let dive_info = TokenInfo {
            token_address: dive_contract_address,
            symbol: "DIVE".to_string(),
        };

        network_manager.add_network(
            "localhost".to_string(),
            vec![token_info1, token_info2, dive_info],
        );
    }

    let dive_info = TokenInfo {
        token_address: dive_contract_address,
        symbol: "DIVE".to_string(),
    };

    network_manager.add_network(name.clone(), vec![dive_info]);

    return Config {
        name,
        endpoint,
        owshen_contract_address,
        owshen_contract_abi,
        dive_contract_address,
        erc20_abi: dive_contract.abi().clone(),
        token_contracts: network_manager,
        poseidon_contract_address: poseidon4_addr.clone(),
    };
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let wallet_path = home::home_dir().unwrap().join(".owshen-wallet.json");
    let config_path = home::home_dir().unwrap().join(".config-wallet.json");

    log::info!(
        "{} {}",
        "Your wallet path:".bright_green(),
        wallet_path.to_string_lossy()
    );

    let opt = OwshenCliOpt::from_args();

    match opt {
        OwshenCliOpt::Init(InitOpt { db, mnemonic }) => {
            let wallet_path = db.unwrap_or(wallet_path.clone());
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();
            if wallet.is_none() {
                let wallet = initialize_wallet(mnemonic).await;
                std::fs::write(wallet_path, serde_json::to_string(&wallet).unwrap()).unwrap();
            } else {
                println!("Wallet is already initialized!");
            }
        }
        OwshenCliOpt::Deploy(DeployOpt {
            endpoint,
            from,
            name,
            config,
            test,
            id,
            deploy_dive,
            deploy_hash_function,
            deploy_owshen,
            genesis,
        }) => {
            let config_path = config.unwrap_or(config_path.clone());
            let config: Option<Config> = std::fs::read_to_string(&config_path)
                .map(|s| {
                    let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
                    c
                })
                .ok();

            let config = initialize_config(
                endpoint,
                from.unwrap_or_default(),
                name,
                test,
                id,
                config,
                deploy_dive,
                deploy_hash_function,
                deploy_owshen,
                genesis,
            )
            .await;
            std::fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
        }
        OwshenCliOpt::Wallet(WalletOpt {
            db,
            config,
            port,
            test,
        }) => {
            let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());
            let config_path = if test {
                println!("here");
                config.unwrap_or_else(|| config_path.clone())
            } else {
                println!("or rhere");
                PathBuf::from(format!("{}/usr/share/networks/Sepolia.json", app_dir_path))
            };
            let wallet_path = db.unwrap_or(wallet_path.clone());
            println!("config path {:?}", config_path);
            let wallet = std::fs::read_to_string(&wallet_path)
                .map(|s| {
                    let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
                    w
                })
                .ok();

            let config = std::fs::read_to_string(&config_path)
                .map(|s| {
                    let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
                    c
                })
                .ok();

            if let (Some(wallet), Some(config)) = (&wallet, &config) {
                let priv_key = wallet.entropy.clone().into();
                let pub_key = PublicKey::from(priv_key);

                serve_wallet(
                    port,
                    priv_key,
                    pub_key,
                    config.token_contracts.clone(),
                    test,
                    config.clone(),
                )
                .await?;
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

async fn deploy(
    client: Arc<Provider<Http>>,
    abi: &str,
    bytecode: &str,
    private_key: SecretKey<Secp256k1>,
    from_address: H160, // Use private key instead of from address
    is_test: bool,
    chain_id: u64,
) -> H160 {
    let wallet = wallet::from(private_key).with_chain_id(chain_id);
    let client_with_signer = SignerMiddleware::new(client, wallet);

    let abi = serde_json::from_str::<Abi>(abi).unwrap();
    let bytecode = Bytes::from_str(bytecode).unwrap();

    let factory = ContractFactory::new(abi, bytecode, Arc::new(client_with_signer));

    let contract = if is_test {
        factory.deploy(()).unwrap().legacy().send().await.unwrap()
    } else {
        let mut deployer = factory.deploy(()).unwrap().legacy();
        deployer.tx.set_from(from_address);
        deployer.send().await.unwrap()
    };

    contract.address()
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
