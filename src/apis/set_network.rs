use crate::{Context, SetNetworkRequest, SetNetworkResponse};

use axum::{extract::Query, Json};
use ethers::providers::{Http, Provider};
use std::sync::{Arc, Mutex};

pub async fn set_network(
    Query(req): Query<SetNetworkRequest>,
    ctx: Arc<Mutex<Context>>,
) -> Result<Json<SetNetworkResponse>, eyre::Report> {
    let provider_url = req.provider_url;
    let provider = Arc::new(Provider::<Http>::try_from(provider_url)?);
    let mut ctx = ctx.lock().unwrap();
    ctx.provider = Some(provider);
    // reset the current coins with last provider
    ctx.coins.clear();
    Ok(Json(SetNetworkResponse {
        success: crate::Empty { ok: true },
    }))
}
