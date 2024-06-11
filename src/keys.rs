use crate::fp::Fp;
use crate::hash::hash4;
use bip39::Mnemonic;
use ff::{Field, PrimeField, PrimeFieldBits};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{Num, Zero};
use rand::Rng;
use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::{
    fmt,
    ops::{Add, Mul, Neg, Sub},
    str::FromStr,
};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Point {
    pub x: Fp,
    pub y: Fp,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PrivateKey {
    pub secret: Fp,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PublicKey {
    pub point: Point,
}
#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
pub struct Entropy {
    pub value: [u8; 16],
}

pub struct EphemeralPrivKey {
    pub secret: Fp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct EphemeralPubKey {
    pub point: Point,
}

impl EphemeralPrivKey {
    pub fn shared_secret(&self, pk: PublicKey) -> Fp {
        let shared_secret = pk.point * self.secret;
        hash4([shared_secret.x, shared_secret.y, 0.into(), 0.into()])
    }
}

struct PublicKeyStr;

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
    pub static ref ORDER: BigUint = BigUint::from_str(
        "2736030358979909402780800718157159386076813972158567259200215660948447373041"
    )
    .unwrap();
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
    #[allow(dead_code)]
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

impl From<PrivateKey> for PublicKey {
    fn from(sk: PrivateKey) -> Self {
        Self {
            point: *BASE * sk.secret,
        }
    }
}

impl PublicKey {
    pub fn derive(&self, r: Fp) -> (EphemeralPrivKey, EphemeralPubKey, PublicKey) {
        let ephemeral = *BASE * r;
        let shared_secret = self.point * r;
        let shared_secret_hash = hash4([shared_secret.x, shared_secret.y, 0.into(), 0.into()]);
        let pub_key = self.point + *BASE * shared_secret_hash;
        (
            EphemeralPrivKey { secret: r },
            EphemeralPubKey { point: ephemeral },
            Self { point: pub_key },
        )
    }

    pub fn derive_random<R: Rng>(
        &self,
        rng: &mut R,
    ) -> (EphemeralPrivKey, EphemeralPubKey, PublicKey) {
        self.derive(Fp::random(rng))
    }
}

impl Entropy {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        Self { value: rng.gen() }
    }

    pub fn to_mnemonic(&self) -> Result<String, bip39::Error> {
        let mnemonic: Mnemonic = Mnemonic::from_entropy(&self.value)?;
        let words: Vec<&str> = mnemonic.word_iter().collect::<Vec<&str>>();
        let phrase: String = words.join(" ");

        Ok(phrase)
    }

    pub fn from_mnemonic(mnemonic: Mnemonic) -> Entropy {
        Entropy {
            value: mnemonic.to_entropy().try_into().unwrap(),
        }
    }
}

impl PrivateKey {
    #[allow(dead_code)]
    pub fn generate<R: Rng>(_rng: &mut R) -> Self {
        let rnd: BigUint = rand::thread_rng().gen_biguint_range(&BigUint::zero(), &*ORDER);
        Self {
            secret: Fp::from_str_vartime(rnd.to_string().as_str()).unwrap(),
        }
    }

    #[allow(dead_code)]
    pub fn to_mnemonic(&self) -> Result<String, bip39::Error> {
        let secret_bytes: Vec<u8> = self.secret.to_repr().as_ref().to_vec();
        let mnemonic: Mnemonic = Mnemonic::from_entropy(&secret_bytes)?;
        let words: Vec<&str> = mnemonic.word_iter().collect::<Vec<&str>>();
        let phrase: String = words.join(" ");

        Ok(phrase)
    }

    pub fn shared_secret(&self, eph: EphemeralPubKey) -> Fp {
        let shared_secret = eph.point * self.secret;
        hash4([shared_secret.x, shared_secret.y, 0.into(), 0.into()])
    }

    pub fn derive(&self, eph: EphemeralPubKey) -> Self {
        let shared_secret = self.shared_secret(eph);
        let secret = BigUint::from_bytes_le(self.secret.to_repr().as_ref());
        let shared_secret = BigUint::from_bytes_le(shared_secret.to_repr().as_ref());
        let stealth_secret = Fp::from_str_vartime(
            ((secret + shared_secret) % ORDER.clone())
                .to_string()
                .as_str(),
        )
        .unwrap();
        Self {
            secret: stealth_secret,
        }
    }

    pub fn nullifier(&self, index: u32) -> Fp {
        hash4([self.secret, Fp::from(index as u64), 0.into(), 0.into()])
    }
}

impl From<Entropy> for PrivateKey {
    fn from(entropy: Entropy) -> Self {
        let mnemonic: Mnemonic = Mnemonic::from_entropy(&entropy.value).unwrap();
        let seed = mnemonic.to_seed("");
        let secret = Fp::from_bytes(&seed);
        PrivateKey { secret }
    }
}

impl FromStr for PublicKey {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("OoOo") {
            return Err(eyre::Report::msg(
                "Invalid Owshen address, address should start with OoOo!",
            ));
        }
        if s.len() != 69 {
            return Err(eyre::Report::msg(
                "Invalid Owshen address, incorrect length!",
            ));
        }
        if let Some(x) = Fp::from_str_vartime(&BigUint::from_str_radix(&s[5..], 16)?.to_string()) {
            let is_odd = if &s[4..5] == "3" {
                true
            } else if &s[4..5] == "2" {
                false
            } else {
                return Err(eyre::Report::msg("Invalid Owshen address!"));
            };
            let div = Option::<Fp>::from((*D * x * x - Fp::ONE).invert())
                .ok_or(eyre::Report::msg("Invalid point!"))?;
            let mut y = Option::<Fp>::from(((*A * x * x - Fp::ONE) * div).sqrt())
                .ok_or(eyre::Report::msg("Invalid point!"))?;
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

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyStr)
    }
}

impl<'de> Visitor<'de> for PublicKeyStr {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expecting a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        PublicKey::from_str(s).map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth() {
        let master_priv_key = PrivateKey {
            secret: Fp::from_str_vartime(
                "2399676232724823934106751350900953157674194292910175666859294040926337260522",
            )
            .unwrap(),
        };
        let master_pub_key: PublicKey = master_priv_key.into();
        let (_, stealth_eph_pub_key, stealth_pub_key) =
            master_pub_key.derive_random(&mut rand::thread_rng());
        assert!(master_pub_key != stealth_pub_key);
        let stealth_priv_key = master_priv_key.derive(stealth_eph_pub_key);
        assert_eq!(PublicKey::from(stealth_priv_key), stealth_pub_key);
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
