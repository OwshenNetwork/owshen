use crate::fp::Fp;
use crate::poseidon::poseidon2;
use crate::poseidon::poseidon4;

pub fn hash4(vals: [Fp; 4]) -> Fp {
    poseidon4(vals)
}

pub fn hash2(vals: [Fp; 2]) -> Fp {
    poseidon2(vals)
}

#[cfg(test)]
mod tests {
    use crate::fp::Fp;
    use crate::hash::{hash4, hash2};
    use ff::PrimeField;
    #[test]
    fn poseidon4_hash() {
        let out: Fp = hash4([Fp::from(0), Fp::from(0), Fp::from(0), Fp::from(0)]);
        let expected = Fp::from_str_vartime(
            "2351654555892372227640888372176282444150254868378439619268573230312091195718",
        )
        .unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn poseidon2_hash() {
        let out: Fp = hash2([Fp::from(0), Fp::from(0)]);
        let expected = Fp::from_str_vartime(
            "14744269619966411208579211824598458697587494354926760081771325075741142829156",
        )
        .unwrap();
        assert_eq!(out, expected);

        let out: Fp = hash2([Fp::from(12), Fp::from(25)]);
        let expected = Fp::from_str_vartime(
            "735578451865327331166453566339024572252777155190980449985765129345128651721",
        )
        .unwrap();
        assert_eq!(out, expected);
    }
}
