use crate::fp::Fp;
use crate::poseidon::poseidon;
use crate::poseidon2::poseidon2;

pub fn hash4(vals: [Fp; 4]) -> Fp {
    poseidon(vals)
}

pub fn hash2(vals: [Fp; 2]) -> Fp {
    poseidon2(vals)
}

#[cfg(test)]
mod tests {
    use crate::fp::Fp;
    use crate::hash::hash4;
    use ff::PrimeField;
    #[test]
    fn poseidon_hash() {
        let out: Fp = hash4([Fp::from(0), Fp::from(0), Fp::from(0), Fp::from(0)]);
        let expected = Fp::from_str_vartime(
            "2351654555892372227640888372176282444150254868378439619268573230312091195718",
        )
        .unwrap();
        assert_eq!(out, expected);
    }
}
