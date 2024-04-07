use std::{collections::HashMap, str::FromStr, sync::Arc};

use bindings::owshen::{SentFilter, SpendFilter};
use ethers::{abi::Abi, prelude::*, types::H160};
use serde::{Deserialize, Serialize};

use crate::{
    genesis::Genesis,
    keys::{Entropy, PrivateKey, PublicKey},
    tree::{self, SparseMerkleTree},
};

pub const GOERLI_ENDPOINT: &str = "https://ethereum-goerli.publicnode.com";
pub const NODE_UPDATE_INTERVAL: u64 = 5;

pub struct Context {
    pub coins: Vec<Coin>,
    pub tree: SparseMerkleTree,
    pub node_manager: NodeManager,
    pub events_latest_status: EventsLatestStatus,
    pub genesis: Genesis,
    pub syncing: Arc<std::sync::Mutex<Option<f32>>>,
    pub syncing_task: Option<
        tokio::task::JoinHandle<
            std::result::Result<(tree::SparseMerkleTree, Vec<Coin>), eyre::Report>,
        >,
    >,
}

pub struct NodeContext {
    pub node_manager: NodeManager,

    pub spent_events: Vec<SpendFilter>,
    pub sent_events: Vec<SentFilter>,
    pub currnet_block_number: u64,
}

#[derive(Clone, Debug)]
pub struct Network {
    pub provider: Arc<Provider<Http>>,
    pub config: Config,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub ip: String,
    pub port: u16,

    pub current_block: u64,
}

impl FromStr for Peer {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let ip = parts.next().ok_or_else(|| eyre::eyre!("Invalid ip"))?;
        let port = parts
            .next()
            .ok_or_else(|| eyre::eyre!("Invalid port"))?
            .parse()?;
        Ok(Peer {
            ip: ip.to_string(),
            port,
            current_block: 0,
        })
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port
    }
}

#[derive(Clone, Debug)]
pub struct EventsLatestStatus {
    pub last_sent_event: u64,
    pub last_spent_event: u64,
}

#[derive(Clone, Debug)]
pub struct NodeManager {
    pub ip: Option<String>,
    pub port: Option<u16>,

    pub network: Option<Network>,
    pub peers: Vec<Peer>,
    pub elected_peer: Option<Peer>,
    pub is_peer2peer: bool,

    pub is_client: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkManager {
    pub networks: HashMap<String, Vec<TokenInfo>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCache {
    pub coins: Vec<Coin>,
    pub tree: SparseMerkleTree,
    pub height: u64,
}
