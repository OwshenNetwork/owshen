use crate::{keys::PublicKey, GetInfoResponse, NetworkManager};

use axum::Json;
use ethers::{abi::Abi, types::H160};

pub async fn info(
    address: PublicKey,
    dive_contract: H160,
    owshen_contract: H160,
    token_contracts: NetworkManager,
    owshen_abi: Abi,
    erc20_abi: Abi,
    is_test: bool,
) -> Result<Json<GetInfoResponse>, eyre::Report> {
    Ok(Json(GetInfoResponse {
        address,
        dive_contract,
        erc20_abi,
        owshen_contract,
        owshen_abi,
        token_contracts,
        is_test,
    }))
}
