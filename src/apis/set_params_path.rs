use crate::config::Wallet;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetParamsPathRequest {
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetParamsPathResponse {}

pub async fn set_params_path(
    Query(req): Query<SetParamsPathRequest>,
    wallet_path: PathBuf,
) -> Result<Json<SetParamsPathResponse>, eyre::Report> {
    let mut wallet = std::fs::read_to_string(&wallet_path).map(|s| {
        let c: Wallet = serde_json::from_str(&s).expect("Invalid config file!");
        c
    })?;
    wallet.params = Some(req.path.into());
    std::fs::write(wallet_path, serde_json::to_string(&wallet)?)?;
    Ok(Json(SetParamsPathResponse {}))
}
