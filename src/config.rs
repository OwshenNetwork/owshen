use std::{collections::HashMap, str::FromStr, sync::Arc};

use ethers::{abi::Abi, prelude::*, types::H160};
use serde::{Deserialize, Serialize};

use crate::{
    genesis::Genesis,
    keys::{Entropy, PrivateKey, PublicKey},
    tree::{self, SparseMerkleTree},
};

pub const GOERLI_ENDPOINT: &str = "https://ethereum-goerli.publicnode.com";

#[derive(Clone, Debug)]
pub struct Network {
    pub provider: Arc<Provider<Http>>,
    pub config: Config,
}

pub struct Context {
    pub coins: Vec<Coin>,
    pub tree: SparseMerkleTree,
    pub network: Option<Network>,
    pub genesis: Genesis,
    pub syncing: Arc<std::sync::Mutex<Option<f32>>>,
    pub syncing_task: Option<
        tokio::task::JoinHandle<
            std::result::Result<(tree::SparseMerkleTree, Vec<Coin>), eyre::Report>,
        >,
    >,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TokenInfo {
    pub token_address: H160,
    pub symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkManager {
    pub networks: HashMap<String, Vec<TokenInfo>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub entropy: Entropy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub endpoint: String,
    pub dive_contract_address: H160,
    pub owshen_contract_address: H160,
    pub owshen_contract_deployment_block_number: U64,
    pub owshen_contract_abi: Abi,
    pub erc20_abi: Abi,
    pub token_contracts: NetworkManager,
    pub poseidon_contract_address: H160,
}

impl NetworkManager {
    pub fn new() -> NetworkManager {
        let mut networks: HashMap<String, Vec<TokenInfo>> = HashMap::new();

        networks.insert(
            "Goerli".to_string(),
            vec![TokenInfo {
                token_address: H160::from_str("0xdD69DB25F6D620A7baD3023c5d32761D353D3De9")
                    .unwrap(),
                symbol: "WETH".to_string(),
            }],
        );

        NetworkManager { networks }
    }

    // pub fn set(&mut self, data: HashMap<String, Vec<TokenInfo>>, expand: bool) {
    //     if expand {
    //         self.networks.extend(data);
    //     } else {
    //         self.networks = data;
    //     }
    // }

    pub fn add_network(&mut self, network: String, token_info: Vec<TokenInfo>) {
        self.networks.insert(network, token_info);
    }

    // pub fn get(&self, network: &str) -> Option<&Vec<TokenInfo>> {
    //     self.networks.get(network)
    // }

    // pub fn has(&self, network: &str, symbol: &str) -> bool {
    //     self.get(network).map_or(false, |tokens| {
    //         tokens.iter().any(|token_info| token_info.symbol == symbol)
    //     })
    // }
}
