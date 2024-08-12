use anyhow::anyhow;
use anyhow::Result;
use ff::PrimeField;
use num_integer::Integer;
use serde::{Deserialize, Serialize};

#[derive(PrimeField, Serialize, Deserialize)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fp([u64; 4]);

use std::ops::*;
use std::str::FromStr;

use ff::{Field, PrimeFieldBits};
use num_bigint::BigUint;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct PointCompressed(pub Fp, pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct PointAffine(pub Fp, pub Fp);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointProjective(pub Fp, pub Fp, pub Fp);

impl PointAffine {
    fn add_assign(&mut self, other: &PointAffine) -> Result<()> {
        if *self == *other {
            *self = self.double()?;
            return Ok(());
        }
        let xx = Option::<Fp>::from((Fp::ONE + *D * self.0 * other.0 * self.1 * other.1).invert())
            .ok_or(anyhow!("Cannot invert"))?;
        let yy = Option::<Fp>::from((Fp::ONE - *D * self.0 * other.0 * self.1 * other.1).invert())
            .ok_or(anyhow!("Cannot invert"))?;
        *self = Self(
            (self.0 * other.1 + self.1 * other.0) * xx,
            (self.1 * other.1 - *A * self.0 * other.0) * yy,
        );
        Ok(())
    }
}

impl PointAffine {
    pub fn is_on_curve(&self) -> bool {
        self.1 * self.1 + *A * self.0 * self.0 == Fp::ONE + *D * self.0 * self.0 * self.1 * self.1
    }
    pub fn is_infinity(&self) -> bool {
        self.0.is_zero().into() && (self.1 == Fp::ONE || self.1 == -Fp::ONE)
    }
    pub fn zero() -> Self {
        Self(Fp::ZERO, Fp::ONE)
    }
    pub fn double(&self) -> Result<Self> {
        let xx = Option::<Fp>::from((*A * self.0 * self.0 + self.1 * self.1).invert())
            .ok_or(anyhow!("Cannot invert"))?;
        let yy = Option::<Fp>::from(
            (Fp::ONE + Fp::ONE - *A * self.0 * self.0 - self.1 * self.1).invert(),
        )
        .ok_or(anyhow!("Cannot invert"))?;
        Ok(Self(
            ((self.0 * self.1) * xx).double(),
            (self.1 * self.1 - *A * self.0 * self.0) * yy,
        ))
    }
    pub fn multiply(&self, scalar: &Fp) -> Result<Self> {
        let mut result = PointProjective::zero();
        let self_proj = self.to_projective();
        for bit in scalar.to_le_bits().iter().rev() {
            result = result.double();
            if *bit {
                result.add_assign(&self_proj)?;
            }
        }
        result.to_affine()
    }
    pub fn to_projective(self) -> PointProjective {
        PointProjective(self.0, self.1, Fp::ONE)
    }
    pub fn compress(&self) -> PointCompressed {
        PointCompressed(self.0, self.1.is_odd().into())
    }
}

impl PointCompressed {
    pub fn decompress(&self) -> Result<PointAffine> {
        let inv = Option::<Fp>::from((Fp::ONE - *D * self.0.square()).invert())
            .ok_or(anyhow!("Cannot invert"))?;
        let mut y = Option::<Fp>::from((inv * (Fp::ONE - *A * self.0.square())).sqrt())
            .ok_or(anyhow!("Cannot take sqrt"))?;
        let is_odd: bool = y.is_odd().into();
        if self.1 != is_odd {
            y = y.neg();
        }
        Ok(PointAffine(self.0, y))
    }
    pub fn verify(&self, message: Fp, sig: &Signature) -> Result<bool> {
        let pk = self.decompress()?;

        if !pk.is_on_curve() || !sig.r.is_on_curve() {
            return Ok(false);
        }

        // h=H(R,A,M)
        let h = hash(&[sig.r.0, sig.r.1, pk.0, pk.1, message]);

        let sb = BASE.multiply(&sig.s)?;

        let mut r_plus_ha = pk.multiply(&h)?;
        r_plus_ha.add_assign(&sig.r)?;

        Ok(r_plus_ha == sb)
    }
}

impl PointProjective {
    fn add_assign(&mut self, other: &PointProjective) -> Result<()> {
        if self.is_zero() {
            *self = *other;
            return Ok(());
        }
        if other.is_zero() {
            return Ok(());
        }
        if self.to_affine()? == other.to_affine()? {
            *self = self.double();
            return Ok(());
        }
        let a = self.2 * other.2; // A = Z1 * Z2
        let b = a.square(); // B = A^2
        let c = self.0 * other.0; // C = X1 * X2
        let d = self.1 * other.1; // D = Y1 * Y2
        let e = *D * c * d; // E = dC · D
        let f = b - e; // F = B − E
        let g = b + e; // G = B + E
        self.0 = a * f * ((self.0 + self.1) * (other.0 + other.1) - c - d);
        self.1 = a * g * (d - *A * c);
        self.2 = f * g;
        Ok(())
    }
}

impl PointProjective {
    pub fn zero() -> Self {
        PointProjective(Fp::ZERO, Fp::ONE, Fp::ZERO)
    }
    pub fn is_zero(&self) -> bool {
        self.2.is_zero().into()
    }
    pub fn double(&self) -> PointProjective {
        if self.is_zero() {
            return PointProjective::zero();
        }
        let b = (self.0 + self.1).square();
        let c = self.0.square();
        let d = self.1.square();
        let e = *A * c;
        let f = e + d;
        let h = self.2.square();
        let j = f - h.double();
        PointProjective((b - c - d) * j, f * (e - d), f * j)
    }
    pub fn to_affine(self) -> Result<PointAffine> {
        if self.is_zero() {
            return Ok(PointAffine::zero());
        }
        let zinv = Option::<Fp>::from(self.2.invert()).ok_or(anyhow!("Cannot invert"))?;
        Ok(PointAffine(self.0 * zinv, self.1 * zinv))
    }
}

lazy_static::lazy_static! {
    pub static ref A: Fp = Fp::from(168700);
    pub static ref D: Fp = Fp::from(168696);
    pub static ref BASE: PointAffine = PointAffine(
        Fp::from_str_vartime(
            "5299619240641551281634865583518297030282874472190772894086521144482721001553"
        )
        .unwrap(),
        Fp::from_str_vartime("16950150798460657717958625567821834550301663161624707787222815936182638968203").unwrap()
    );
    pub static ref BASE_COFACTOR: PointAffine = BASE.multiply(&Fp::from(8)).unwrap();
    pub static ref ORDER: BigUint = BigUint::from_str(
        "21888242871839275222246405745257275088614511777268538073601725287587578984328"
    )
    .unwrap();
}

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Signature {
    pub r: PointAffine,
    pub s: Fp,
}

pub struct PrivateKey(Fp);

fn hash(inp: &[Fp]) -> Fp {
    inp.iter().fold(Fp::ONE, |a, b| a * b)
}

impl PrivateKey {
    fn to_pub(&self) -> Result<PointCompressed> {
        Ok(BASE.multiply(&self.0)?.compress())
    }
    fn sign(&self, randomness: Fp, message: Fp) -> Result<Signature> {
        let pk = self.to_pub()?.decompress()?;

        // r=H(b,M)
        let r = hash(&[randomness, message]);

        // R=rB
        let rr = BASE.multiply(&r)?;

        // h=H(R,A,M)
        let h = hash(&[rr.0, rr.1, pk.0, pk.1, message]);

        // s = (r + ha) mod ORDER
        let mut s = BigUint::from_bytes_le(r.to_repr().as_ref());
        let mut ha = BigUint::from_bytes_le(h.to_repr().as_ref());
        ha.mul_assign(&BigUint::from_bytes_le(self.0.to_repr().as_ref()));
        s.add_assign(&ha);
        s = s.mod_floor(&*ORDER);
        let s_as_fr = {
            let s_bytes = s.to_bytes_le();
            let mut s_repr = FpRepr([0u8; 32]);
            s_repr.0[0..s_bytes.len()].copy_from_slice(&s_bytes);
            Option::<Fp>::from(Fp::from_repr(s_repr)).ok_or(anyhow!("Invalid repr"))?
        };

        Ok(Signature { r: rr, s: s_as_fr })
    }
}
