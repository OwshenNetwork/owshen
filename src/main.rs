mod apis;
mod commands;
mod config;
mod fp;
mod genesis;
mod hash;
mod helper;
mod keys;
mod poseidon;
mod proof;
mod tree;

use bindings::owshen::Point as OwshenPoint;
use colored::Colorize;
use config::{Coin, Config, NetworkManager, GOERLI_ENDPOINT};
use ethers::{abi::Abi, prelude::*, types::H160};
use eyre::Result;
use helper::{h160_to_u256, u256_to_h160};
use keys::Point;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tree::SparseMerkleTree;

#[macro_use]
extern crate lazy_static;

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Init(commands::InitOpt),
    Info(commands::InfoOpt),
    Wallet(commands::WalletOpt),
    Deploy(commands::DeployOpt),
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: String::new(),
            endpoint: GOERLI_ENDPOINT.to_string(),
            dive_contract_address: H160::default(),
            owshen_contract_address: H160::default(),
            owshen_contract_deployment_block_number: U64::default(),
            owshen_contract_abi: Abi::default(),
            erc20_abi: Abi::default(),
            token_contracts: NetworkManager::new(),
            poseidon_contract_address: H160::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletCache {
    coins: Vec<Coin>,
    tree: SparseMerkleTree,
    height: u64,
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
        OwshenCliOpt::Init(init_opt) => {
            commands::init(init_opt, wallet_path).await;
        }
        OwshenCliOpt::Deploy(deploy_opt) => {
            commands::deploy(deploy_opt, config_path).await;
        }
        OwshenCliOpt::Wallet(wallet_opt) => {
            commands::wallet(wallet_opt, config_path, wallet_path).await;
        }
        OwshenCliOpt::Info(info_opt) => {
            commands::info(info_opt, wallet_path).await;
        }
    }

    Ok(())
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
