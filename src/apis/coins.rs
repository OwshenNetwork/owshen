use axum::{extract, response::Json, routing::get, Router};
use bindings::owshen::{Owshen, Point as OwshenPoint, SentFilter, SpendFilter};
use bindings::simple_erc_20::SimpleErc20;
use std::net::SocketAddr;
use tokio::time::timeout;

use crate::hash::hash4;
use tower_http::cors::CorsLayer;

use ethers::prelude::*;

use crate::keys::{EphemeralKey, PrivateKey, PublicKey};
use eyre::Result;

use crate::tree::SparseMerkleTree;
use crate::u256_to_h160;
use crate::Coin;
use crate::GetCoinsResponse;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;

use crate::fp::Fp;
use crate::keys::Point;
use crate::Context;
use ff::PrimeField;
use std::path::PathBuf;
use std::sync::Mutex;
use structopt::StructOpt;

pub async fn coins(context: Arc<Mutex<Context>>, contract: Contract<Provider<Http>>) {
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
    .await
    .unwrap();
    for sent_event in sent_events {
        let ephemeral = EphemeralKey {
            point: Point {
                x: Fp::from_str_vartime(&sent_event.ephemeral.x.to_string()).unwrap(),
                y: Fp::from_str_vartime(&sent_event.ephemeral.y.to_string()).unwrap(),
            },
        };

        let stealth_priv = priv_key.derive(ephemeral);
        let stealth_pub: PublicKey = stealth_priv.clone().into();
        let index: U256 = sent_event.index;
        let hint_amount = sent_event.hint_amount;
        let hint_token_address = sent_event.hint_token_address;
        let u64_index: u64 = index.low_u64();
        let commitment = Fp::from_str(&U256::to_string(&sent_event.commitment)).unwrap();
        tree.set(u64_index, commitment);

        let calc_commitment = hash4([
            stealth_pub.point.x,
            stealth_pub.point.y,
            Fp::from_str(&U256::to_string(&hint_amount)).unwrap(),
            Fp::from_str(&U256::to_string(&hint_token_address)).unwrap(),
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
        let amount = U256::to_string(
            &(Fp::from_str(&U256::to_string(&hint_amount)).unwrap() - shared_secret).into(),
        );
        let token_address = U256::to_string(
            &(Fp::from_str(&U256::to_string(&hint_token_address)).unwrap() - shared_secret).into(),
        );

        let calc_commitment_obfuscate = hash4([
            stealth_pub.point.x,
            stealth_pub.point.y,
            Fp::from_str(&amount).unwrap(),
            Fp::from_str(&token_address).unwrap(),
        ]);

        if commitment == calc_commitment_obfuscate {
            println!("I HAVE SOMETHING ");
            my_coins.push(Coin {
                index,
                uint_token: u256_to_h160(U256::from_str(&token_address).unwrap()),
                amount: U256::from_str(&amount).unwrap(),
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
        .await
        .unwrap()
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

    Json(GetCoinsResponse {
        coins: my_coins.clone(),
    })
}
