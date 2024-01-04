use crate::fp::Fp;
use crate::genesis::genesis_events;

use crate::helper::extract_token_amount;
use crate::keys::Point;
use crate::keys::{EphemeralKey, PrivateKey, PublicKey};
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
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::timeout;

pub async fn coins(
    context_coin: Arc<Mutex<Context>>,
    provider: Arc<Provider<Http>>,
    contract: Contract<Provider<Http>>,
    priv_key: PrivateKey,
) -> Result<Json<GetCoinsResponse>, eyre::Report> {
    let wallet_cache_path = home::home_dir().unwrap().join(".owshen-wallet-cache");
    let cache: Option<WalletCache> = if let Ok(f) = std::fs::read(&wallet_cache_path) {
        bincode::deserialize(&f).ok()
    } else {
        None
    };

    let root: U256 = contract.method("root", ())?.call().await?;

    let blk_number = provider.get_block_number().await?.as_u64();
    if let Some(cache) = cache {
        if Into::<U256>::into(cache.tree.root()) == root {
            return Ok(Json(GetCoinsResponse { coins: cache.coins }));
        }
    }

    let mut my_coins: Vec<Coin> = Vec::new();

    let dive_contract_address = context_coin.lock().unwrap().dive_contract_address.clone();

    let mut tree = SparseMerkleTree::new(16);

    let sent_events = timeout(std::time::Duration::from_secs(5), async {
        contract
            .event::<SentFilter>()
            .from_block(0)
            .to_block(1000)
            .address(ValueOrArray::Value(contract.address()))
            .query()
            .await
            .unwrap()
    })
    .await?;

    for sent_event in genesis_events(dive_contract_address)
        .iter()
        .chain(sent_events.iter())
    {
        let ephemeral = EphemeralKey {
            point: Point {
                x: Fp::try_from(sent_event.ephemeral.x)?,
                y: Fp::try_from(sent_event.ephemeral.y)?,
            },
        };

        let stealth_priv = priv_key.derive(ephemeral);
        let stealth_pub: PublicKey = stealth_priv.clone().into();
        let index: U256 = sent_event.index;
        let hint_amount = sent_event.hint_amount;
        let hint_token_address = sent_event.hint_token_address;
        let u64_index: u64 = index.low_u64();
        let commitment = Fp::try_from(sent_event.commitment)?;
        let shared_secret = stealth_priv.shared_secret(ephemeral);
        tree.set(u64_index, commitment);

        match extract_token_amount(
            hint_token_address,
            hint_amount,
            shared_secret,
            commitment,
            stealth_pub,
        ) {
            Ok(Some((fp_hint_token_address, fp_hint_amount))) => {
                println!("I HAVE SOMETHING");
                my_coins.push(Coin {
                    index,
                    uint_token: u256_to_h160(fp_hint_token_address.into()),
                    amount: fp_hint_amount.into(),
                    nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                    priv_key: stealth_priv,
                    pub_key: stealth_pub,
                    commitment: sent_event.commitment,
                });
            }
            Ok(None) => {
                println!("No coin was found");
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    for spend_event in contract
        .event::<SpendFilter>()
        .from_block(0)
        .to_block(100)
        .query()
        .await?
    {
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

            println!(
                "YOU SPEND YOUR DEPOSIT! nullifier:{:?}",
                spend_event.nullifier
            );
        }
    }

    let wallet_cache = WalletCache {
        coins: my_coins.clone(),
        tree: tree.clone(),
        height: blk_number,
    };

    std::fs::write(&wallet_cache_path, bincode::serialize(&wallet_cache)?)?;

    let mut ctx = context_coin.lock().unwrap();
    ctx.coins = my_coins.clone();
    ctx.tree = tree;

    Ok(Json(GetCoinsResponse {
        coins: my_coins.clone(),
    }))
}
