pub mod babyjubjub;
use babyjubjub::*;

use alloy::primitives::Address;
use anyhow::Result;

use crate::blockchain::Blockchain;

pub fn owshen_airdrop<B: Blockchain>(
    _chain: &mut B,
    by: Address,
    owshen_address: PointCompressed,
    owshen_sig: Signature,
) -> Result<()> {
    log::info!("Someone is claiming his owshen airdrop, by {}!", by);
    owshen_address.verify(Fp::from(123), &owshen_sig)?;
    Ok(())
}
