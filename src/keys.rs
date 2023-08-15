use crate::fp::Fp;
use crate::hash::hash;
use ff::Field;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct PrivateKey {
    secret: Fp,
}

impl PrivateKey {
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
