use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::keys::{Point, PublicKey};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStealthRequest {
    address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetStealthResponse {
    address: Point,
    ephemeral: Point,
}

pub async fn stealth(
    Query(req): Query<GetStealthRequest>,
) -> Result<Json<GetStealthResponse>, eyre::Report> {
    let pub_key = PublicKey::from_str(&req.address)?;
    let (_, ephemeral, address) = pub_key.derive_random(&mut rand::thread_rng());
    Ok(Json(GetStealthResponse {
        address: address.point,
        ephemeral: ephemeral.point,
    }))
}
