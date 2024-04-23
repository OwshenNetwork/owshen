use std::{path::PathBuf, str::FromStr, sync::Arc};

use bindings::{dive_token::DiveToken, owshen::Owshen, simple_erc_20::SimpleErc20};
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
    config::{Config, NetworkManager, TokenInfo},
    genesis,
};

#[derive(StructOpt, Debug)]
pub struct DeployOpt {
    #[structopt(long)]
    from: Option<String>,
    #[structopt(long)]
    endpoint: String,
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    config: PathBuf,
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

pub async fn deploy(opt: DeployOpt) -> Result<(), eyre::Report> {
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

    let cfg: Option<Config> = std::fs::read_to_string(&config)
        .map(|s| {
            let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
            c
        })
        .ok();

    let cfg = initialize_config(
        endpoint,
        from.unwrap_or_default(),
        name,
        test,
        id,
        cfg,
        deploy_dive,
        deploy_hash_function,
        deploy_owshen,
        genesis,
    )
    .await?;

    std::fs::write(config, serde_json::to_string(&cfg)?)?;

    Ok(())
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
) -> Result<Config, eyre::Report> {
    let mut network_manager = NetworkManager::new();
    let provider = Arc::new(Provider::<Http>::try_from(endpoint.clone())?);
    let private_key_bytes = hex_decode(&from)?;
    let private_key: SecretKey<_> = SecretKey::from_slice(&private_key_bytes)?;
    let chain_id: u64 = chain_id.parse()?;
    let wallet = wallet::from(private_key.clone()).with_chain_id(chain_id);

    let from_address = if is_test {
        let accounts = provider.get_accounts().await?;
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
        .await?
    } else {
        if let Some(c) = &config {
            c.poseidon4_contract_address
        } else {
            return Err(eyre::eyre!("No config file!"));
        }
    };
    log::info!("Poseidon4 contract address {:?}", poseidon4_addr);

    let poseidon2_addr = if deploy_hash_function {
        log::info!("Deploying hash function...");
        deploy_codes(
            provider.clone(),
            include_str!("../assets/poseidon2.abi"),
            include_str!("../assets/poseidon2.evm"),
            private_key.clone(),
            from_address,
            is_test,
            chain_id,
        )
        .await?
    } else {
        if let Some(c) = &config {
            c.poseidon2_contract_address
        } else {
            return Err(eyre::eyre!("No config file!"));
        }
    };
    log::info!("Poseidon2 contract address {:?}", poseidon2_addr);

    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let nonce = provider.get_transaction_count(from_address, None).await?;

    log::info!("Deployer address: {:?}", from_address);
    let dive_contract_address = if deploy_dive {
        log::info!("Deploying DIVE token...");
        DiveToken::deploy(
            client.clone(),
            (U256::from_str_radix("89900000000000000000000", 10).unwrap(),),
        )?
        .legacy()
        .from(from_address)
        .nonce(nonce)
        .send()
        .await?
        .address()
    } else {
        if let Some(c) = &config {
            c.dive_contract_address
        } else {
            panic!("No config file!");
        }
    };
    log::info!("DIVE token address {:?}", dive_contract_address);

    let dive_contract = DiveToken::new(dive_contract_address, client.clone());

    let genesis = if genesis_feed {
        log::info!("Filling the genesis tree... (This might take some time)");
        let genesis = genesis::fill_genesis(dive_contract_address);
        std::fs::write("owshen-genesis.dat", bincode::serialize(&genesis)?)?;
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

    let new_nonce = provider.get_transaction_count(from_address, None).await?;

    log::info!("Getting Owshen contract deployment blocknumber...");
    let mut owshen_contract_deployment_block_number: U64 = U64::default();

    let (owshen_contract_address, owshen_contract_abi) = if deploy_owshen {
        log::info!("Deploying Owshen contract...");
        let o = Owshen::deploy(
            client,
            (
                poseidon4_addr,
                poseidon2_addr,
                Into::<U256>::into(genesis.chc.get_last_checkpoint()),
                Into::<U256>::into(genesis.chc.size()),
            ),
        )?
        .legacy()
        .from(from_address)
        .nonce(new_nonce)
        .send()
        .await?;

        let owshen_client = o.client();

        owshen_contract_deployment_block_number = owshen_client.get_block_number().await?;

        dive_contract
            .method::<_, bool>("transfer", (o.address(), Into::<U256>::into(genesis.total)))?
            .legacy()
            .from(from_address)
            .send()
            .await?;

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
        let provider = Arc::new(Provider::<Http>::try_from(provider_url)?);
        let accounts = provider.get_accounts().await?;
        let from = accounts[0];
        let test_token = SimpleErc20::deploy(
            provider.clone(),
            (
                U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                "test_token".to_string(),
                "TEST".to_string(),
            ),
        )?
        .legacy()
        .from(from)
        .send()
        .await?;

        let second_test_token = SimpleErc20::deploy(
            provider.clone(),
            (
                U256::from_str_radix("1000000000000000000000", 10).unwrap(),
                "test_token".to_string(),
                "TEST".to_string(),
            ),
        )?
        .legacy()
        .from(from)
        .send()
        .await?;

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

    Ok(Config {
        name,
        endpoint,
        owshen_contract_address,
        owshen_contract_deployment_block_number,
        owshen_contract_abi,
        dive_contract_address,
        erc20_abi: dive_contract.abi().clone(),
        token_contracts: network_manager,
        poseidon4_contract_address: poseidon4_addr,
        poseidon2_contract_address: poseidon2_addr,
    })
}

async fn deploy_codes(
    client: Arc<Provider<Http>>,
    abi: &str,
    bytecode: &str,
    private_key: SecretKey<Secp256k1>,
    from_address: H160, // Use private key instead of from address
    is_test: bool,
    chain_id: u64,
) -> Result<H160, eyre::Report> {
    let wallet = wallet::from(private_key).with_chain_id(chain_id);
    let client_with_signer = SignerMiddleware::new(client, wallet);

    let abi = serde_json::from_str::<Abi>(abi)?;
    let bytecode = Bytes::from_str(bytecode)?;

    let factory = ContractFactory::new(abi, bytecode, Arc::new(client_with_signer));

    let contract = if is_test {
        factory.deploy(())?.legacy().send().await?
    } else {
        let mut deployer = factory.deploy(())?.legacy();
        deployer.tx.set_from(from_address);
        deployer.send().await?
    };

    Ok(contract.address())
}
