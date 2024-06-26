use std::path::PathBuf;

use crate::config::Wallet;
use crate::keys::Entropy;
use axum::Json;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PostInitRequest {
    Generate,
    Import { words: Vec<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PostInitResponse {
    Generated { words: Vec<String> },
    Imported,
}

pub async fn init(
    wallet_path: PathBuf,
    Json(req): Json<PostInitRequest>,
) -> Result<Json<PostInitResponse>, eyre::Report> {
    let (entropy, resp): (Entropy, PostInitResponse) =
        if let PostInitRequest::Import { words } = req {
            (
                Entropy::from_mnemonic(words.join(" ").parse()?),
                PostInitResponse::Imported,
            )
        } else {
            let ent = Entropy::generate(&mut thread_rng());
            (
                ent,
                PostInitResponse::Generated {
                    words: ent.to_mnemonic().into_iter().collect(),
                },
            )
        };
    std::fs::write(
        wallet_path,
        serde_json::to_string(&Wallet {
            entropy,
            params: None,
            burnt_addresses: vec![],
            burnt_coins: vec![],
        })?,
    )?;
    Ok(Json(resp))
}
