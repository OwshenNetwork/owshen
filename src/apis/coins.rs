use crate::fp::Fp;
use crate::helper::extract_token_amount;
use crate::keys::Point;
use crate::keys::{EphemeralPubKey, PrivateKey, PublicKey};
use crate::tree::SparseMerkleTree;
use crate::u256_to_h160;
use crate::Coin;
use crate::Context;
use crate::GetCoinsResponse;
use crate::WalletCache;

use axum::Json;
use bindings::owshen::{SentFilter, SpendFilter};
use ethers::prelude::*;
use eyre::Result;
use rayon::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::timeout;

pub async fn coins(
    provider: Arc<Mutex<Context>>,
    priv_key: PrivateKey,
    owshen_contract_deployment_block_number: U64,
) -> Result<Json<GetCoinsResponse>, eyre::Report> {
    let mut prov = provider.lock().await;

    if let Some(sync_task) = &prov.syncing_task {
        if sync_task.is_finished() {
            let task = prov.syncing_task.take().unwrap();
            let (tree, coins) = task.await??;
            prov.tree = tree;
            prov.coins = coins;
            *prov.syncing.lock().unwrap() = None;
        } else {
            return Ok(Json(GetCoinsResponse {
                coins: vec![],
                syncing: Some(prov.syncing.lock().unwrap().unwrap_or_default()),
            }));
        }
    }

    let network = prov.network.clone();

    if let Some(network) = network {
        let contract: ContractInstance<Arc<_>, _> = Contract::new(
            network.config.owshen_contract_address,
            network.config.owshen_contract_abi,
            Arc::clone(&network.provider),
        );
        let curr_block_number = network.provider.get_block_number().await?.as_u64();
        let wallet_cache_path = home::home_dir().unwrap().join(".owshen-wallet-cache");
        let cache: Option<WalletCache> = if let Ok(f) = std::fs::read(&wallet_cache_path) {
            bincode::deserialize(&f).ok()
        } else {
            None
        };
        const UPDATE_THRESHOLD: u64 = 5;

        let root: U256 = contract.method("root", ())?.call().await?;
        if let Some(cache) = &cache {
            if Into::<U256>::into(cache.tree.root()) == root
                && curr_block_number.wrapping_sub(cache.height as u64) < UPDATE_THRESHOLD
            {
                prov.coins = cache.coins.clone();
                prov.tree = cache.tree.clone();
                return Ok(Json(GetCoinsResponse {
                    coins: cache.coins.clone(),
                    syncing: None,
                }));
            }
        }

        let tree = cache
            .as_ref()
            .map(|c| c.tree.clone())
            .unwrap_or(SparseMerkleTree::new(16));

        let syncing_arc = Arc::new(std::sync::Mutex::new(Some(0f32)));
        prov.syncing = syncing_arc.clone();

        let mut step = 1024;
        let mut curr = if let Some(cache) = &cache {
            if cache.height > step {
                (cache.height as u64).wrapping_sub(step)
            } else {
                cache.height
            }
        } else {
            owshen_contract_deployment_block_number.as_u64()
        };
        let mut spent_events = Vec::new();
        let mut sent_events = Vec::new();

        while curr < curr_block_number {
            log::info!("Loading events from blocks {} to {}...", curr, curr + step);
            if let Some((new_spent_events, new_sent_events)) =
                timeout(std::time::Duration::from_secs(10), async {
                    contract
                        .event::<SpendFilter>()
                        .from_block(curr)
                        .to_block(curr + step)
                        .address(ValueOrArray::Value(contract.address()))
                        .query()
                        .await
                })
                .await
                .map(|r| r.ok())
                .ok()
                .unwrap_or_default()
                .zip(
                    timeout(std::time::Duration::from_secs(10), async {
                        contract
                            .event::<SentFilter>()
                            .from_block(curr)
                            .to_block(curr + step)
                            .address(ValueOrArray::Value(contract.address()))
                            .query()
                            .await
                    })
                    .await
                    .map(|r| r.ok())
                    .ok()
                    .unwrap_or_default(),
                )
            {
                spent_events.extend(new_spent_events);
                sent_events.extend(new_sent_events);
                curr += step;
                if step < 1024 {
                    step = step * 2;
                }
            } else {
                step = step / 2;
            }
        }

        let is_genesis_processed = cache.is_some();

        let all_events = if is_genesis_processed {
            sent_events
        } else {
            prov.genesis
                .events
                .iter()
                .cloned()
                .map(|e| e.into())
                .chain(sent_events.into_iter())
                .collect::<Vec<_>>()
        };

        let mut tree_task = tree.clone();
        let mut my_coins: Vec<Coin> = cache.map(|c| c.coins).unwrap_or_default();

        let task = tokio::task::spawn_blocking(move || {
            let mut cnt = 0;
            for chunk in all_events.chunks(128) {
                let progress = (cnt as f32) / all_events.len() as f32;
                log::info!(
                    "Processing events {}-{} of {}... ({}%)\r",
                    cnt,
                    cnt + chunk.len(),
                    all_events.len(),
                    (progress * 100.0) as u32
                );
                *syncing_arc.lock().unwrap() = Some(progress);
                for e in chunk.iter() {
                    tree_task.set(e.index.low_u64(), Fp::try_from(e.commitment)?);
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
                        let stealth_priv = priv_key.derive(ephemeral);
                        let stealth_pub: PublicKey = stealth_priv.clone().into();
                        let index: U256 = sent_event.index;
                        let hint_amount = sent_event.hint_amount;
                        let hint_token_address = sent_event.hint_token_address;
                        let commitment = Fp::try_from(sent_event.commitment).ok()?;
                        let shared_secret = stealth_priv.shared_secret(ephemeral);
                        match extract_token_amount(
                            hint_token_address,
                            hint_amount,
                            shared_secret,
                            commitment,
                            stealth_pub,
                        ) {
                            Ok(Some((fp_hint_token_address, fp_hint_amount))) => Some(Coin {
                                index,
                                uint_token: u256_to_h160(fp_hint_token_address.into()),
                                amount: fp_hint_amount.into(),
                                nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                                priv_key: stealth_priv,
                                pub_key: stealth_pub,
                                commitment: sent_event.commitment,
                            }),
                            Ok(None) => None,
                            Err(_) => None,
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
                for _coin in my_coins.clone() {
                    let coin_position = my_coins
                        .iter()
                        .position(|_coin| _coin.nullifier == spend_event.nullifier);
                    match coin_position {
                        Some(index) => {
                            my_coins.remove(index);
                        }
                        None => {}
                    }
                }
            }

            let wallet_cache = WalletCache {
                coins: my_coins.clone(),
                tree: tree_task.clone(),
                height: curr_block_number as u64,
            };
            std::fs::write(&wallet_cache_path, bincode::serialize(&wallet_cache)?)?;
            Ok::<(SparseMerkleTree, Vec<Coin>), eyre::Report>((tree_task, my_coins))
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
