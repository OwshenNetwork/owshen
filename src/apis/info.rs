use crate::{keys::PublicKey, GetInfoResponse, NetworkManager};
use crate::{Context, Network};

use axum::Json;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn info(
    address: PublicKey,
    info_context: Arc<Mutex<Context>>,
    token_contracts: NetworkManager,
    is_test: bool,
) -> Result<Json<GetInfoResponse>, eyre::Report> {
    let info_arc: Option<Network> = info_context.lock().await.network.clone();
    if let Some(network) = info_arc {
        Ok(Json(GetInfoResponse {
            address,
            dive_contract: network.config.dive_contract_address,
            erc20_abi: network.config.erc20_abi,
            owshen_contract: network.config.owshen_contract_address,
            owshen_abi: network.config.owshen_contract_abi,
            token_contracts,
            is_test,
        }))
    } else {
        panic!("Provider is not set");
    }
}
