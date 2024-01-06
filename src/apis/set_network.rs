use crate::{Config, Context, Network, SetNetworkRequest, SetNetworkResponse};

use axum::{extract::Query, Json};
use ethers::providers::{Http, Provider};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

lazy_static! {
    static ref NETWORK_CONFIG_MAP: HashMap<String, String> =
        [("http://127.0.0.1:8545".into(), "localhost.json".into())]
            .into_iter()
            .collect();
}

pub async fn set_network(
    Query(req): Query<SetNetworkRequest>,
    ctx: Arc<Mutex<Context>>,
) -> Result<Json<SetNetworkResponse>, eyre::Report> {
    let provider_url = req.provider_url;
    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(provider_url.clone())?);
    let mut ctx = ctx.lock().unwrap();

    let config_path = NETWORK_CONFIG_MAP.get(&provider_url).unwrap().clone();
    let config = std::fs::read_to_string(&config_path)
        .map(|s| {
            let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
            c
        })
        .ok()
        .unwrap();

    ctx.network = Some(Network { provider, config });
    // reset the current coins with last provider
    ctx.coins.clear();
    Ok(Json(SetNetworkResponse {
        success: crate::Empty { ok: true },
    }))
}
