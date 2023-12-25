use axum::Json;
use bindings::owshen::{SentFilter, SpendFilter};
use ethers::prelude::*;
use eyre::Result;

use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::timeout;

use crate::fp::Fp;
use crate::hash::hash4;
use crate::keys::Point;
use crate::keys::{EphemeralKey, PrivateKey, PublicKey};
use crate::tree::SparseMerkleTree;
use crate::u256_to_h160;
use crate::Coin;
use crate::Context;
use crate::GetCoinsResponse;

#[allow(dead_code)]
pub async fn coins(
    context_coin: Arc<Mutex<Context>>,
    contract: Contract<Provider<Http>>,
    priv_key: PrivateKey,
) -> Result<Json<GetCoinsResponse>, eyre::Report> {
    let mut my_coins: Vec<Coin> = Vec::new();
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
    for sent_event in sent_events {
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
        tree.set(u64_index, commitment);

        let calc_commitment = hash4([
            stealth_pub.point.x,
            stealth_pub.point.y,
            Fp::try_from(hint_amount)?,
            Fp::try_from(hint_token_address)?,
        ]);

        let shared_secret = stealth_priv.shared_secret(ephemeral);

        if commitment == calc_commitment {
            println!("ITS MINE");
            my_coins.push(Coin {
                index,
                uint_token: u256_to_h160(hint_token_address),
                amount: sent_event.hint_amount,
                nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                priv_key: stealth_priv,
                pub_key: stealth_pub,
                commitment: sent_event.commitment,
            });
        }

        // get sends
        let amount = U256::to_string(&(Fp::try_from(hint_amount)? - shared_secret).into());
        let token_address =
            U256::to_string(&(Fp::try_from(hint_token_address)? - shared_secret).into());

        let calc_commitment_obfuscate = hash4([
            stealth_pub.point.x,
            stealth_pub.point.y,
            Fp::from_str(&amount)?,
            Fp::from_str(&token_address)?,
        ]);

        if commitment == calc_commitment_obfuscate {
            println!("I HAVE SOMETHING ");
            my_coins.push(Coin {
                index,
                uint_token: u256_to_h160(U256::from_str(&token_address)?),
                amount: U256::from_str(&amount)?,
                nullifier: stealth_priv.nullifier(index.low_u32()).into(),
                priv_key: stealth_priv,
                pub_key: stealth_pub,
                commitment: commitment.into(),
            });
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
    let mut ctx = context_coin.lock().unwrap();
    ctx.coins = my_coins.clone();
    ctx.tree = tree;

    Ok(Json(GetCoinsResponse {
        coins: my_coins.clone(),
    }))
}
