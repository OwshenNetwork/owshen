mod genesis_data;

use crate::fp::Fp;
use crate::h160_to_u256;
use crate::hash::hash4;
use crate::keys::PublicKey;
use crate::SparseMerkleTree;
use bindings::owshen::SentFilter;
use ethers::types::H160;
use ethers::types::U256;
use ff::{Field, PrimeField};

use genesis_data::GENESIS;

pub fn genesis_events(dive_token_address: H160) -> Vec<SentFilter> {
    let coeff = Fp::from_str_vartime("1000000000000000000").unwrap();
    GENESIS
        .into_iter()
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
            SentFilter {
                ephemeral: eph.point.into(),
                index: U256::from(i),
                timestamp: U256::from(0),
                hint_amount: amount.into(),
                hint_token_address: h160_to_u256(dive_token_address),
                commitment: commit.into(),
            }
        })
        .collect()
}

pub fn fill_genesis(smt: &mut SparseMerkleTree, dive_token_address: H160) -> U256 {
    let mut total_amount: U256 = U256::default();
    for event in genesis_events(dive_token_address).into_iter() {
        smt.set(event.index.low_u64(), event.commitment.try_into().unwrap());
        total_amount += event.hint_amount;
    }
    total_amount
}
