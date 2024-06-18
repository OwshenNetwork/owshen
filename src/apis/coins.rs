use crate::checkpointed_hashchain::CheckpointedHashchain;
use crate::config::{Context, WalletCache};
use crate::fp::Fp;
use crate::helper::extract_token_amount;
use crate::keys::Point;
use crate::keys::{EphemeralPubKey, PrivateKey, PublicKey};
use crate::u256_to_h160;
use crate::Coin;

use axum::Json;
use bindings::owshen::{SentFilter, SpendFilter};
use ethers::prelude::*;
use eyre::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GetCoinsResponse {
    coins: Vec<Coin>,
    syncing: Option<f32>,
}

pub async fn coins(
    provider: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
) -> Result<Json<GetCoinsResponse>, eyre::Report> {
    let mut prov = provider.lock().await;

    if let Some(sync_task) = &prov.syncing_task {
        if sync_task.is_finished() {
            let task = prov.syncing_task.take().unwrap();
            let (chc, coins) = task.await??;
            prov.chc = chc;
            prov.coins = coins;
            *prov.syncing.lock().unwrap() = None;
        } else {
            return Ok(Json(GetCoinsResponse {
                coins: vec![],
                syncing: Some(prov.syncing.lock().unwrap().unwrap_or_default()),
            }));
        }
    }

    let network = prov.node_manager.get_provider_network();

    if let Some(network) = network {
        let contract: ContractInstance<Arc<_>, _> = Contract::new(
            network.config.owshen_contract_address,
            network.config.owshen_contract_abi,
            Arc::clone(&network.provider),
        );
        let curr_block_number = network.provider.get_block_number().await?.as_u64();
        let wallet_cache_path = home::home_dir()
            .ok_or(eyre::eyre!("Cannot extract home directory!"))?
            .join(".owshen-wallet-cache");
        let cache: Option<WalletCache> = if let Ok(f) = std::fs::read(&wallet_cache_path) {
            log::info!("Cache exists");
            bincode::deserialize(&f).ok()
        } else {
            log::info!("Cache not found");
            None
        };
        const UPDATE_THRESHOLD: u64 = 128;

        // let root: U256 = contract.method("root", ())?.call().await?;
        let contract_chc_state: (U256, U256) = contract.method("getState", ())?.call().await?;
        let (head, checkpoint) = prov.chc.get_state();
        let genesis_checkpoint_head: U256 = prov.genesis.chc.get_state().1.into();
        log::info!("Genesis Checkpoint head {}", genesis_checkpoint_head);

        if let Some(cache) = &cache {
            let (head_u256, checkpoint_u256) = (head.into(), checkpoint.into());
            log::info!(
                "Local/Contract heads: {}/{}",
                head_u256,
                contract_chc_state.0
            );
            log::info!(
                "Local/Contract checkpoints: {}/{}",
                checkpoint_u256,
                contract_chc_state.1
            );
            log::info!("chc size: {}", prov.chc.size());
            if contract_chc_state == (head_u256, checkpoint_u256)
                && curr_block_number.wrapping_sub(cache.height as u64) < UPDATE_THRESHOLD
            {
                prov.coins = cache.coins.clone();
                prov.chc = cache.chc.clone();
                return Ok(Json(GetCoinsResponse {
                    coins: cache.coins.clone(),
                    syncing: None,
                }));
            }
        }

        let chc = match cache.clone() {
            Some(cache) => cache.chc.clone(),
            None => prov.genesis.chc.clone(),
        };

        let syncing_arc = Arc::new(std::sync::Mutex::new(Some(0f32)));
        prov.syncing = syncing_arc.clone();

        let step = 1024;
        let mut curr = if let Some(cache) = &cache {
            log::info!("Cache height: {}", cache.height as u64);
            if cache.height > step {
                (cache.height as u64).wrapping_sub(step)
            } else {
                cache.height
            }
        } else {
            network
                .config
                .owshen_contract_deployment_block_number
                .as_u64()
        };
        curr = curr.max(
            network
                .config
                .owshen_contract_deployment_block_number
                .as_u64(),
        );
        curr = curr - 1;

        #[allow(unused_assignments)]
        let mut spent_events: Vec<SpendFilter> = vec![];

        #[allow(unused_assignments)]
        let mut sent_events: Vec<SentFilter> = vec![];

        if prov.node_manager.is_peer2peer {
            let from_spent = prov.events_latest_status.last_spent_event;
            let from_sent = prov.events_latest_status.last_sent_event;
            let (tmp_spent_events, tmp_sent_events, _) = prov
                .node_manager
                .clone()
                .get_events_from_elected_peer(from_spent, from_sent)
                .await?;
            spent_events = tmp_spent_events;
            sent_events = tmp_sent_events;
        } else {
            spent_events = prov
                .node_manager
                .get_spend_events(curr, curr_block_number)
                .await;
            sent_events = prov
                .node_manager
                .get_sent_events(curr, curr_block_number)
                .await;

            log::info!(
                "{}-{}, Got {} sent events and {} spent events",
                curr,
                curr_block_number,
                sent_events.len(),
                spent_events.len()
            );
        }

        let chc_task = chc.clone();
        let my_coins: Vec<Coin> = cache.map(|c| c.coins).unwrap_or_default();

        let genesis_events: Vec<SentFilter> = prov
            .genesis
            .events
            .iter()
            .cloned()
            .map(|e| e.into())
            .collect();
        let task = tokio::task::spawn_blocking(move || {
            sync_coins(
                &genesis_events,
                chc_task,
                &priv_key,
                sent_events,
                spent_events,
                my_coins,
                curr_block_number,
                &wallet_cache_path,
                &syncing_arc,
            )
        });

        prov.syncing_task = Some(task);

        Ok(Json(GetCoinsResponse {
            coins: vec![],
            syncing: Some(0f32),
        }))
    } else {
        Ok(Json(GetCoinsResponse {
            coins: vec![],
            syncing: None,
        }))
    }
}

fn sync_coins(
    genesis_events: &Vec<SentFilter>,
    mut chc_task: CheckpointedHashchain,
    priv_key: &PrivateKey,
    sent_events: Vec<SentFilter>,
    spent_events: Vec<SpendFilter>,
    mut my_coins: Vec<Coin>,
    curr_block_number: u64,
    wallet_cache_path: &std::path::PathBuf,
    syncing_arc: &Arc<std::sync::Mutex<Option<f32>>>,
) -> Result<(CheckpointedHashchain, Vec<Coin>), eyre::Report> {
    let cache_exists = std::fs::metadata(&wallet_cache_path).is_ok();

    log::info!(
        "Genesis len {}, Sent events len{}, Genesis events len + Sent event len {}",
        genesis_events.len(),
        sent_events.len(),
        genesis_events.len() + sent_events.len()
    );
    let mut vec_sent_event = sent_events.to_vec();
    if !cache_exists {
        vec_sent_event = genesis_events.clone();
        vec_sent_event.extend(sent_events);
    }
    let sent_events = vec_sent_event;

    let mut cnt = 0;
    for chunk in sent_events.chunks(128) {
        let progress = (cnt as f32) / sent_events.len() as f32;
        log::info!(
            "Processing events {}-{} of {}... ({}%)\r",
            cnt,
            cnt + chunk.len(),
            sent_events.len(),
            (progress * 100.0) as u32
        );
        *syncing_arc.lock().unwrap() = Some(progress);
        for e in chunk.iter() {
            chc_task.set(e.index.as_u64(), Fp::try_from(e.commitment)?);
            log::debug!(
                "Setting {}th CHC element to event with memo '{}' (New checkpoint/head: {:?})",
                e.index.as_u64(),
                e.memo,
                chc_task.get_state()
            );
        }
        for new_coin in chunk
            .par_iter()
            .filter_map(|sent_event| {
                let ephemeral = EphemeralPubKey {
                    point: Point {
                        x: Fp::try_from(sent_event.ephemeral.x).ok()?,
                        y: Fp::try_from(sent_event.ephemeral.y).ok()?,
                    },
                };
                // hash(g^sr) + s
                let stealth_priv = priv_key.derive(ephemeral);

                // g^(hash(g^sr) + s)
                let stealth_pub: PublicKey = stealth_priv.clone().into();
                let index: U256 = sent_event.index;
                let hint_amount = sent_event.hint_amount;
                let hint_token_address = sent_event.hint_token_address;
                let commitment = Fp::try_from(sent_event.commitment).ok()?;
                let shared_secret = stealth_priv.shared_secret(ephemeral);

                if let Ok(Some((fp_hint_token_address, fp_hint_amount))) = extract_token_amount(
                    hint_token_address,
                    hint_amount,
                    shared_secret,
                    commitment,
                    stealth_pub,
                ) {
                    log::debug!(
                        "Found coin Index {} - Amount: {:?}",
                        index.low_u32(),
                        fp_hint_amount
                    );
                    Some(Coin {
                        index,
                        uint_token: u256_to_h160(fp_hint_token_address.into()),
                        amount: fp_hint_amount.into(),
                        nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                        priv_key: stealth_priv,
                        pub_key: stealth_pub,
                        commitment: sent_event.commitment,
                        memo: sent_event.memo.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
        {
            if !my_coins.iter().any(|c| c.index == new_coin.index) {
                my_coins.push(new_coin);
            }
        }
        cnt += chunk.len();
    }

    for spend_event in spent_events {
        my_coins.retain(|c| c.nullifier != spend_event.nullifier);
    }

    log::info!(
        "Wallet cache current block number {}",
        curr_block_number as u64
    );

    let wallet_cache = WalletCache {
        coins: my_coins.clone(),
        chc: chc_task.clone(),
        height: curr_block_number as u64,
    };
    std::fs::write(&wallet_cache_path, bincode::serialize(&wallet_cache)?)?;
    Ok::<(CheckpointedHashchain, Vec<Coin>), eyre::Report>((chc_task.clone(), my_coins.clone()))
}
