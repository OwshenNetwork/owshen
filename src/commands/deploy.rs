use std::{path::PathBuf, str::FromStr, sync::Arc};

use bindings::{owshen::Owshen, simple_erc_20::SimpleErc20};
use ethers::{
    abi::Abi,
    core::k256::{elliptic_curve::SecretKey, Secp256k1},
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet as wallet},
};
use hex::decode as hex_decode;
use structopt::StructOpt;

use crate::{
    config::{Config, NetworkManager, TokenInfo, GOERLI_ENDPOINT},
    genesis,
};

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

pub async fn deploy(opt: DeployOpt, config_path: PathBuf) {
    let DeployOpt {
        from,
        endpoint,
        name,
        config,
        test,
        id,
        deploy_dive,
        deploy_hash_function,
        deploy_owshen,
        genesis,
    } = opt;

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
        deploy_codes(
            provider.clone(),
            include_str!("../assets/poseidon4.abi"),
            include_str!("../assets/poseidon4.evm"),
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

    let erc20_abi = serde_json::from_str::<Abi>(include_str!("../assets/erc20.abi")).unwrap();
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

    log::info!("Getting Owshen contract deployment blocknumber...");
    let mut owshen_contract_deployment_block_number: U64 = U64::default();

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

        let owshen_client = o.client();

        owshen_contract_deployment_block_number = owshen_client.get_block_number().await.unwrap();

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
            "Localhost".to_string(),
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
        owshen_contract_deployment_block_number,
        owshen_contract_abi,
        dive_contract_address,
        erc20_abi: dive_contract.abi().clone(),
        token_contracts: network_manager,
        poseidon_contract_address: poseidon4_addr.clone(),
    };
}

async fn deploy_codes(
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
