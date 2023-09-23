use crate::fp::Fp;
use crate::hash::hash;
use ff::{Field, PrimeField, PrimeFieldBits};
use num_bigint::BigUint;
use num_traits::Num;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Point {
    pub x: Fp,
    pub y: Fp,
}

lazy_static! {
    pub static ref INF: Point = Point {
        x: 0.into(),
        y: 1.into(),
    };
    pub static ref BASE: Point = {
        let base2 = *G + *G;
        let base4 = base2 + base2;
        let base8 = base4 + base4;
        base8
    };
    pub static ref G: Point = Point {
        x: Fp::from_str_vartime(
            "995203441582195749578291179787384436505546430278305826713579947235728471134"
        )
        .unwrap(),
        y: Fp::from_str_vartime(
            "5472060717959818805561601436314318772137091100104008585924551046643952123905"
        )
        .unwrap(),
    };
    pub static ref A: Fp = 168700.into();
    pub static ref D: Fp = 168696.into();
}

impl Point {
    pub fn is_on_curve(&self) -> bool {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        *A * x2 + y2 == Fp::ONE + *D * x2 * y2
    }
}

impl Neg for Point {
    type Output = Self;
    fn neg(self) -> Self {
        Point {
            x: -self.x,
            y: self.y,
        }
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let common = *D * self.x * self.y * other.x * other.y;
        let x_div = (Fp::ONE + common).invert().unwrap();
        let y_div = (Fp::ONE - common).invert().unwrap();
        let x = (self.x * other.y + self.y * other.x) * x_div;
        let y = (self.y * other.y - *A * self.x * other.x) * y_div;
        Point { x, y }
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl Mul<Fp> for Point {
    type Output = Self;

    fn mul(self, other: Fp) -> Self {
        let mut result = *INF;
        for bit in other.to_le_bits().iter().rev() {
            result = result + result;
            if *bit {
                result = result + self;
            }
        }
        result
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PrivateKey {
    pub secret: Fp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PublicKey {
    pub point: Point,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EphemeralKey {
    pub point: Point,
}

impl From<PrivateKey> for PublicKey {
    fn from(sk: PrivateKey) -> Self {
        Self {
            point: *BASE * sk.secret,
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Cipher {
    a: Point,
    b: Point,
}

impl PublicKey {
    pub fn encrypt(&self, random: Fp, msg: Point) -> Cipher {
        Cipher {
            a: *BASE * random,
            b: msg + self.point * random,
        }
    }

    pub fn derive<R: Rng>(&self, rng: &mut R) -> (EphemeralKey, PublicKey) {
        let r = Fp::random(rng);
        let ephemeral = *BASE * r;
        let shared_secret = self.point * r;
        let shared_secret_hash = hash(shared_secret.x, shared_secret.y);
        let pub_key = self.point + *BASE * shared_secret_hash;
        (EphemeralKey { point: ephemeral }, Self { point: pub_key })
    }
}

impl PrivateKey {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            secret: Fp::random(rng),
        }
    }
    pub fn decrypt(&self, cipher: Cipher) -> Point {
        cipher.b - cipher.a * self.secret
    }
    pub fn nullifier(&self, index: u32) -> Fp {
        hash(self.secret, Fp::from(index as u64))
    }
}

impl FromStr for PublicKey {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 69 || !s.starts_with("OoOo") {
            return Err(eyre::Report::msg("Invalid Owshen address!"));
        }
        if let Some(x) = Fp::from_str_vartime(&BigUint::from_str_radix(&s[5..], 16)?.to_string()) {
            let is_odd = if &s[4..5] == "3" {
                true
            } else if &s[4..5] == "2" {
                false
            } else {
                return Err(eyre::Report::msg("Invalid Owshen address!"));
            };
            let div = (*D * x * x - Fp::ONE).invert().unwrap();
            let mut y = ((*A * x * x - Fp::ONE) * div).sqrt().unwrap();
            if Into::<bool>::into(y.is_odd()) != is_odd {
                y = -y;
            }
            Ok(Self {
                point: Point { x, y },
            })
        } else {
            Err(eyre::Report::msg("Invalid Owshen address!"))
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OoOo")?;
        let is_odd: bool = self.point.y.is_odd().into();
        write!(f, "{}", if is_odd { "3" } else { "2" })?;
        for byte in self.point.x.to_repr().as_ref().iter().rev() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt() {
        let priv_key = PrivateKey {
            secret: 23456.into(),
        };
        let pub_key: PublicKey = priv_key.into();
        let msg = G.mul(123456.into());
        let rnd: Fp = 987654.into();
        let enc = pub_key.encrypt(rnd, msg);
        let dec = priv_key.decrypt(enc);

        assert_eq!(dec, msg);
    }

    #[test]
    fn test_generator() {
        assert!(G.is_on_curve());
    }

    #[test]
    fn test_inf() {
        assert!(INF.is_on_curve());
        assert_eq!(*G + *INF, *G);
        assert_eq!(*G, *G + *INF);
    }

    #[test]
    fn test_add() {
        assert_eq!((*G + *G) + *G, *G + (*G + *G));
    }

    #[test]
    fn test_mul() {
        let g5_sum = *G + *G + *G + *G + *G;
        let g5_mul = *G * Fp::from(5);
        assert_eq!(g5_sum, g5_mul);
        let g6_sum = *G + *G + *G + *G + *G + *G;
        let g6_mul = *G * Fp::from(6);
        assert_eq!(g6_sum, g6_mul);
    }
}
