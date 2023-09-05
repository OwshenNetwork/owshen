use crate::fp::Fp;
use crate::hash::hash;
use ff::{Field, PrimeField};
use num_bigint::BigUint;
use num_traits::Num;
use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PrivateKey {
    secret: Fp,
}

impl PrivateKey {
    pub fn from_secret(secret: Fp ) -> Self {
        Self{secret}
    }
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            secret: Fp::random(rng),
        }
    }
    pub fn nullifier(&self) -> Fp {
        hash(self.secret, Fp::from(2))
    }
}

#[derive(Debug, Clone)]
pub struct PublicKey {
    commitment: Fp,
}

impl From<PrivateKey> for PublicKey {
    fn from(sk: PrivateKey) -> Self {
        Self {
            commitment: hash(sk.secret, Fp::from(1)),
        }
    }
}

impl FromStr for PublicKey {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 68 || !s.starts_with("OoOo") {
            return Err(eyre::Report::msg("Invalid Owshen address!"));
        }
        if let Some(commitment) =
            Fp::from_str_vartime(&BigUint::from_str_radix(&s[4..], 16)?.to_string())
        {
            Ok(Self { commitment })
        } else {
            Err(eyre::Report::msg("Invalid Owshen address!"))
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OoOo")?;
        for byte in self.commitment.to_repr().as_ref().iter().rev() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
