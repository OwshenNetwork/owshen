mod genesis_data;

use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::EphemeralKey;
use crate::keys::PublicKey;
use crate::SparseMerkleTree;
use bindings::owshen::SentFilter;
use ethers::types::H160;
use ethers::types::U256;
use ff::{Field, PrimeField};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use genesis_data::GENESIS;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    ephemeral: EphemeralKey,
    index: usize,
    timestamp: u64,
    hint_amount: Fp,
    hint_token_address: Fp,
    commitment: Fp,
}

impl Into<SentFilter> for Entry {
    fn into(self) -> SentFilter {
        SentFilter {
            ephemeral: self.ephemeral.point.into(),
            index: U256::from(self.index),
            timestamp: U256::from(self.timestamp),
            hint_amount: self.hint_amount.into(),
            hint_token_address: self.hint_token_address.into(),
            commitment: self.commitment.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis {
    pub total: Fp,
    pub smt: SparseMerkleTree,
    pub events: Vec<Entry>,
}

unsafe impl Send for Genesis {}
unsafe impl Sync for Genesis {}

pub fn gen_genesis_events(dive_token_address: H160) -> Vec<Entry> {
    let dive_token_addr: Fp = h160_to_u256(dive_token_address).try_into().unwrap();
    let coeff = Fp::from_str_vartime("1000000000000000000").unwrap();
    GENESIS
        .into_par_iter()
        .enumerate()
        .map(|(i, (addr, amnt))| {
            let pk: PublicKey = addr.parse().unwrap();
            let (eph, stealth_pub) = pk.derive(Fp::ZERO);
            let amount = Fp::from(amnt) * coeff;
            let commit = hash4([
                stealth_pub.point.x,
                stealth_pub.point.y,
                amount,
                Fp::try_from(h160_to_u256(dive_token_address)).unwrap(),
            ]);
            Entry {
                ephemeral: eph,
                index: i,
                timestamp: 0,
                hint_amount: amount,
                hint_token_address: dive_token_addr,
                commitment: commit,
            }
        })
        .collect()
}

pub fn fill_genesis(depth: usize, dive_token_address: H160) -> Genesis {
    let mut smt = SparseMerkleTree::new(depth);
    let mut total: Fp = Fp::default();
    let events = gen_genesis_events(dive_token_address);
    for event in events.iter() {
        smt.set(event.index as u64, event.commitment.try_into().unwrap());
        total += event.hint_amount;
    }
    Genesis { total, smt, events }
}
