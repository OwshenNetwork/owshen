use crate::{commands::wallet::Mode, config::Context, keys::PublicKey, NetworkManager};

use axum::Json;
use ethers::abi::Abi;
use ethers::types::H160;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    address: PublicKey,
    erc20_abi: Abi,
    dive_contract: H160,
    owshen_contract: H160,
    owshen_abi: Abi,
    token_contracts: NetworkManager,
    is_test: bool,
    mode: Mode,
}

pub async fn info(
    address: PublicKey,
    info_context: Arc<Mutex<Context>>,
    is_test: bool,
    mode: Mode,
) -> Result<Json<GetInfoResponse>, eyre::Report> {
    let network = info_context
        .lock()
        .await
        .node_manager
        .get_provider_network()
        .ok_or(eyre::eyre!("Provider is not set"))?;
    Ok(Json(GetInfoResponse {
        address,
        dive_contract: network.config.dive_contract_address,
        erc20_abi: network.config.erc20_abi,
        owshen_contract: network.config.owshen_contract_address,
        owshen_abi: network.config.owshen_contract_abi,
        token_contracts: network.config.token_contracts,
        is_test,
        mode,
    }))
}
