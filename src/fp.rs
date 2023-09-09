use ethers::prelude::*;
use ff::PrimeField;
use num_bigint::BigUint;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fp([u64; 4]);

impl Into<U256> for Fp {
    fn into(self) -> U256 {
        U256::from_str_radix(
            &BigUint::from_bytes_le(self.to_repr().as_ref()).to_str_radix(16),
            16,
        )
        .unwrap()
    }
}
