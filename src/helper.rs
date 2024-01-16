use crate::fp::Fp;
use crate::hash::hash4;
use crate::keys::PublicKey;
use ethers::types::{H160, U256};

pub fn extract_token_amount(
    hint_token_address: U256,
    hint_amount: U256,
    shared_secret: Fp,
    commitment: Fp,
    stealth_pub: PublicKey,
) -> Result<Option<(Fp, Fp)>, eyre::Report> {
    let fp_hint_token_address = Fp::try_from(hint_token_address)?;
    let fp_hint_amount = Fp::try_from(hint_amount)?;
    let amount = fp_hint_amount - shared_secret;
    let token_address = fp_hint_token_address - shared_secret;

    if hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        amount,
        token_address,
    ]) == commitment
    {
        return Ok(Some((token_address, amount)));
    }

    if hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        fp_hint_amount,
        fp_hint_token_address,
    ]) == commitment
    {
        return Ok(Some((fp_hint_token_address, fp_hint_amount)));
    }

    if hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        amount,
        fp_hint_token_address,
    ]) == commitment
    {
        return Ok(Some((fp_hint_token_address, amount)));
    }

    if hash4([
        stealth_pub.point.x,
        stealth_pub.point.y,
        fp_hint_amount,
        token_address,
    ]) == commitment
    {
        return Ok(Some((token_address, fp_hint_amount)));
    }

    Ok(None)
}

pub fn u256_to_h160(u256: U256) -> H160 {
    let mut bytes: [u8; 32] = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    let address_bytes: &[u8] = &bytes[12..32]; // Taking the last 20 bytes for ethereum address
    H160::from_slice(address_bytes)
}

pub fn h160_to_u256(h160_val: H160) -> U256 {
    let mut bytes = [0u8; 32];
    bytes[12..32].copy_from_slice(h160_val.as_bytes());

    U256::from_big_endian(&bytes)
}
