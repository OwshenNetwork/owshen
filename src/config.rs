use std::{collections::HashMap, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use bindings::owshen::{SentFilter, SpendFilter};
use ethers::{abi::Abi, prelude::*, types::H160};
use serde::{Deserialize, Serialize};

use crate::{
    checkpointed_hashchain::CheckpointedHashchain,
    fp::Fp,
    genesis,
    hash::hash2,
    helper::u256_to_h160,
    keys::{Entropy, PrivateKey, PublicKey},
};

use sha2::{Digest, Sha256};
pub const NODE_UPDATE_INTERVAL: u64 = 5;

pub struct Context {
    pub coins: Vec<Coin>,
    pub chc: CheckpointedHashchain,
    pub node_manager: NodeManager,
    pub events_latest_status: EventsLatestStatus,
    pub syncing: Arc<std::sync::Mutex<Option<f32>>>,
    pub syncing_task: Option<
        tokio::task::JoinHandle<
            std::result::Result<(CheckpointedHashchain, Vec<Coin>), eyre::Report>,
        >,
    >,
}

impl Context {
    pub fn switch_network(&mut self, config: Config) -> Result<(), eyre::Report> {
        if Some(config.chain_id)
            == self
                .node_manager
                .network
                .as_ref()
                .map(|c| c.config.chain_id)
        {
            return Ok(());
        }
        let provider: Arc<Provider<Http>> =
            Arc::new(Provider::<Http>::try_from(config.endpoint.clone())?);
        log::info!("Filling the genesis tree... (This might take some time)");
        let genesis = genesis::fill_genesis(config.dive_contract_address);
        self.node_manager.set_provider_network(Network {
            provider,
            config,
            genesis,
        });
        self.coins.clear();

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwshenSend {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwshenWithdraw {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OwshenTransaction {
    Send(OwshenSend),
    Withdraw(OwshenWithdraw),
}

pub struct NodeContext {
    pub node_manager: NodeManager,

    pub spent_events: Vec<SpendFilter>,
    pub sent_events: Vec<SentFilter>,
    pub currnet_block_number: u64,

    pub mempool: Vec<OwshenTransaction>,
}

#[derive(Clone, Debug)]
pub struct Network {
    pub provider: Arc<Provider<Http>>,
    pub config: Config,
    pub genesis: genesis::Genesis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub addr: SocketAddr,
    pub current_block: u64,
}

impl FromStr for Peer {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr: SocketAddr = s
            .parse()
            .map_err(|_| eyre::eyre!("Invalid socket address"))?;
        Ok(Peer {
            addr,
            current_block: 0,
        })
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

#[derive(Clone, Debug)]
pub struct EventsLatestStatus {
    pub last_sent_event: usize,
    pub last_spent_event: usize,
}

#[derive(Clone, Debug)]
pub struct NodeManager {
    pub external_addr: Option<SocketAddr>,

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
pub struct BurnAddress {
    pub address: H160,
    pub preimage: U256,
    pub used: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurntCoin {
    pub amount: U256,
    pub salt: U256,
    pub encrypted: bool,
}

impl BurntCoin {
    pub fn get_balance(&self) -> U256 {
        if !self.encrypted {
            self.amount
        } else {
            hash2([
                Fp::try_from(self.amount).unwrap(),
                Fp::try_from(self.salt).unwrap(),
            ])
            .into()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub entropy: Entropy,
    pub params: Option<PathBuf>,
    pub burnt_addresses: Vec<BurnAddress>,
    pub burnt_coins: Vec<BurntCoin>,
}

impl Wallet {
    pub fn derive_burn_addr(&self) -> BurnAddress {
        let random_number = rand::random::<u64>().to_string();
        let mut preimage = self.entropy.to_mnemonic().unwrap().to_string() + &random_number;
        let mut hasher = Sha256::new();
        hasher.update(preimage);
        preimage = format!("{:X}", hasher.finalize());
        let reminder = U256::from_str_radix(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            10,
        )
        .unwrap();
        let u256_preimage = U256::from_str(&preimage).unwrap() % reminder;
        let fp_preimage = Fp::try_from(u256_preimage).unwrap();
        let hashed_preimage: U256 = hash2([fp_preimage, fp_preimage]).into();
        let address = u256_to_h160(hashed_preimage);
        BurnAddress {
            address: address,
            preimage: u256_preimage,
            used: Some(false),
        }
    }

    pub fn get_burn_address_info_by_address(&self, address: H160) -> Option<&BurnAddress> {
        self.burnt_addresses.iter().find(|&x| x.address == address)
    }

    pub fn set_used_burn_address(&mut self, address: H160) {
        let index = self
            .burnt_addresses
            .iter()
            .position(|x| x.address == address)
            .unwrap();
        self.burnt_addresses[index].used = Some(true);
    }

    pub fn derive_burnt_coin(&self, amount: U256, encryted: bool) -> BurntCoin {
        let reminder = U256::from_str_radix(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            10,
        )
        .unwrap();
        let random_number: U256 = U256::from(rand::random::<u128>()) % reminder;

        BurntCoin {
            amount: amount,
            salt: random_number,
            encrypted: encryted,
        }
    }

    pub fn save_wallet(&self, path: PathBuf) -> Result<(), eyre::Report> {
        let resp = std::fs::write(path, serde_json::to_string(&self).unwrap());
        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre::eyre!("Error saving wallet: {}", e)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub endpoint: String,
    pub chain_id: u64,
    pub dive_contract_address: H160,
    pub owshen_contract_address: H160,
    pub owshen_contract_deployment_block_number: U64,
    pub owshen_contract_abi: Abi,
    pub erc20_abi: Abi,
    pub token_contracts: NetworkManager,
    pub poseidon4_contract_address: H160,
    pub poseidon2_contract_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCache {
    pub coins: Vec<Coin>,
    pub chc: CheckpointedHashchain,
    pub height: u64,
}
