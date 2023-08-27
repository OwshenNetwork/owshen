use crate::fp::Fp;

#[derive(Debug, Clone)]
struct SparseMerkleTree {}

#[derive(Debug, Clone)]
struct MerkleProof {
    value: Fp,
    proof: Vec<Fp>,
}

impl SparseMerkleTree {
    pub fn new() -> Self {
        Self {}
    }
    pub fn set(&mut self, index: u64, value: Fp) {
        unimplemented!()
    }

    pub fn get(&self, index: u64) -> MerkleProof {
        unimplemented!()
    }

    pub fn root(&self) -> Fp {
        unimplemented!()
    }

    pub fn verify(root: Fp, proof: &MerkleProof) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_merkle_trees() {
        let mut tree = SparseMerkleTree::new();
        tree.set(123, Fp::from(234));
        tree.set(345, Fp::from(456));
        assert_eq!(tree.root(), Fp::from(567));
    }
}
