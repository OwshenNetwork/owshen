use axum::{extract::Query, Json};
use std::str::FromStr;

use crate::keys::PublicKey;
use crate::{GetStealthRequest, GetStealthResponse};

pub async fn stealth(
    Query(req): Query<GetStealthRequest>,
) -> Result<Json<GetStealthResponse>, eyre::Report> {
    let pub_key = PublicKey::from_str(&req.address)?;
    let (ephemeral, address) = pub_key.derive(&mut rand::thread_rng());
    Ok(Json(GetStealthResponse {
        address: address.point,
        ephemeral: ephemeral.point,
    }))
}
