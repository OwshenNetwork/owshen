mod apis;
mod checkpointed_hashchain;
mod commands;
mod config;
mod fp;
mod genesis;
mod hash;
mod helper;
mod keys;
mod network;
mod poseidon;
mod proof;

use bindings::owshen::Point as OwshenPoint;
use colored::Colorize;
use config::{Coin, NetworkManager};
use eyre::Result;
use helper::{h160_to_u256, u256_to_h160};
use keys::Point;
use std::env;
use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;

#[derive(StructOpt, Debug)]
enum OwshenCliOpt {
    Init(commands::InitOpt),
    Info(commands::InfoOpt),
    Wallet(commands::WalletOpt),
    Deploy(commands::DeployOpt),
    Node(commands::NodeOpt),
    Burn(commands::BurnOpt),
    Mint(commands::MintOpt),
    Dive(commands::DiveOpt),
    Participate(commands::ParticipateOpt),
    Claim(commands::ClaimOpt),
    Spend(commands::SpendOpt),
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

    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        args.push("wallet".to_string());
        args.push("--mode".to_string());
        args.push("windows".to_string());
        args.push("--peer2peer".to_string());
        args.push("--bootstrap-peers".to_string());
        args.push("195.35.3.102:8888".to_string());
    }

    let wallet_path = home::home_dir().unwrap().join(".owshen-wallet.json");

    log::info!(
        "{} {}",
        "Your wallet path:".bright_green(),
        wallet_path.to_string_lossy()
    );

    let opt = OwshenCliOpt::from_iter(args.iter());

    match opt {
        OwshenCliOpt::Init(init_opt) => {
            commands::init(init_opt, wallet_path).await?;
        }
        OwshenCliOpt::Deploy(deploy_opt) => {
            commands::deploy(deploy_opt).await?;
        }
        OwshenCliOpt::Wallet(wallet_opt) => {
            commands::wallet(wallet_opt, wallet_path).await?;
        }
        OwshenCliOpt::Info(info_opt) => {
            commands::info(info_opt, wallet_path).await?;
        }
        OwshenCliOpt::Node(node_opt) => {
            commands::node(node_opt).await?;
        }

        OwshenCliOpt::Burn(burn_opt) => {
            commands::burn(burn_opt, wallet_path).await;
        }
        OwshenCliOpt::Mint(mint_opt) => {
            commands::mint(mint_opt, wallet_path).await;
        }
        OwshenCliOpt::Dive(dive_opt) => {
            commands::dive(dive_opt).await;
        }
        OwshenCliOpt::Participate(participate_opt) => {
            commands::participate(participate_opt).await;
        }
        OwshenCliOpt::Claim(claim_opt) => {
            commands::claim(claim_opt).await;
        }
        OwshenCliOpt::Spend(spend_opt) => {
            commands::spend(spend_opt, wallet_path).await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ethers::abi::Abi;
    use ethers::providers::{Http, Provider};
    use ethers::utils::Ganache;

    use std::sync::Arc;

    use ethers::core::types::Bytes;
    use ethers::middleware::contract::ContractFactory;
    use ethers::middleware::Middleware;
    use ethers::types::U256;
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
