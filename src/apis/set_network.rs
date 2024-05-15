use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::config::{Config, Context};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Empty {
    ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetNetworkRequest {
    pub chain_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetNetworkResponse {
    pub success: Empty,
}

lazy_static! {
    static ref NETWORK_CONFIG_MAP: HashMap<u64, Config> = {
        let mut res = HashMap::new();
        for path in std::fs::read_dir("assets/networks").unwrap() {
            let conf: Config =
                serde_json::from_str(&std::fs::read_to_string(path.unwrap().path()).unwrap())
                    .unwrap();
            res.insert(conf.chain_id, conf);
        }
        res
    };
}

pub async fn set_network(
    Query(req): Query<SetNetworkRequest>,
    ctx: Arc<Mutex<Context>>,
    forced_config: Option<Config>,
) -> Result<Json<SetNetworkResponse>, eyre::Report> {
    if forced_config.is_none() {
        let config = NETWORK_CONFIG_MAP
            .get(&req.chain_id.parse().unwrap())
            .ok_or(eyre::eyre!(
                "Unsupported network with chain-id: {}!",
                req.chain_id
            ))?
            .clone();

        ctx.lock().await.switch_network(config)?;
    }

    Ok(Json(SetNetworkResponse {
        success: Empty { ok: true },
    }))
}
