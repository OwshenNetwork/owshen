use std::sync::Arc;

use alloy::primitives::Address;

use crate::genesis::Genesis;

#[derive(Debug, Clone)]

pub struct Config {
    pub chain_id: u64,
    pub owner: Option<Address>,
    pub owshen: Address,
    pub genesis: Arc<Genesis>,
    pub provider_address: reqwest::Url,
}
