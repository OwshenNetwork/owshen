use axum::Json;
use ethers::abi::Abi;
use ethers::types::H160;

use crate::{keys::PublicKey, GetInfoResponse, TokenInfo};

pub async fn info(
    address: PublicKey,
    dive_contract: H160,
    owshen_contract: H160,
    token_contracts: Vec<TokenInfo>,
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
