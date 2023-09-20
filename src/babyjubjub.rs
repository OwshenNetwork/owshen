use crate::fp::Fp;
use ff::{Field, PrimeField, PrimeFieldBits};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct Point {
    pub x: Fp,
    pub y: Fp,
}

lazy_static! {
    static ref INF: Point = Point {
        x: 0.into(),
        y: 1.into(),
    };
    static ref G: Point = Point {
        x: Fp::from_str_vartime(
            "995203441582195749578291179787384436505546430278305826713579947235728471134"
        )
        .unwrap(),
        y: Fp::from_str_vartime(
            "5472060717959818805561601436314318772137091100104008585924551046643952123905"
        )
        .unwrap(),
    };
    static ref A: Fp = 168700.into();
    static ref D: Fp = 168696.into();
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
struct PrivateKey {
    pub secret: Fp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PublicKey {
    pub point: Point,
}

impl From<PrivateKey> for PublicKey {
    fn from(sk: PrivateKey) -> Self {
        Self {
            point: *G * sk.secret,
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
struct Cipher {
    a: Point,
    b: Point,
}

impl PublicKey {
    pub fn encrypt(&self, random: Fp, msg: Point) -> Cipher {
        Cipher {
            a: *G * random,
            b: msg + self.point * random,
        }
    }
}

impl PrivateKey {
    pub fn decrypt(&self, cipher: Cipher) -> Point {
        cipher.b - cipher.a * self.secret
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
