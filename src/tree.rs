use crate::fp::Fp;
use crate::hash::hash;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SparseMerkleTree {
    defaults: Vec<Fp>,
    layers: Vec<HashMap<u64, Fp>>,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub value: Fp,
    pub proof: Vec<Fp>,
}

impl SparseMerkleTree {
    pub fn depth(&self) -> usize {
        self.layers.len() - 1
    }

    pub fn new(depth: usize) -> Self {
        let mut defaults = vec![Fp::from(0)];
        for i in 0..depth {
            defaults.push(hash(defaults[i], defaults[i]));
        }
        Self {
            defaults,
            layers: vec![HashMap::new(); depth + 1],
        }
    }

    fn get_at_layer(&self, layer: usize, index: u64) -> Fp {
        *self.layers[layer]
            .get(&index)
            .unwrap_or(&self.defaults[layer])
    }

    pub fn set(&mut self, mut index: u64, mut value: Fp) {
        for i in 0..self.depth() + 1 {
            self.layers[i].insert(index, value);
            value = if index % 2 == 0 {
                hash(value, self.get_at_layer(i, index + 1))
            } else {
                hash(self.get_at_layer(i, index - 1), value)
            };
            index /= 2;
        }
    }

    pub fn get(&self, mut index: u64) -> MerkleProof {
        let value = self.get_at_layer(0, index);
        let mut proof = vec![];
        for i in 0..self.depth() {
            proof.push(self.get_at_layer(i, if index % 2 == 0 { index + 1 } else { index - 1 }));
            index /= 2;
        }
        MerkleProof { value, proof }
    }

    pub fn root(&self) -> Fp {
        self.get_at_layer(self.depth(), 0)
    }

    pub fn verify(root: Fp, mut index: u64, proof: &MerkleProof) -> bool {
        let mut value = proof.value;
        for p in proof.proof.iter() {
            value = if index % 2 == 0 {
                hash(value, *p)
            } else {
                hash(*p, value)
            };
            index /= 2;
        }
        value == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff::PrimeField;

    #[test]
    fn test_merkle_trees() {
        let mut tree = SparseMerkleTree::new(32);
        tree.set(123, Fp::from(234));
        tree.set(345, Fp::from(456));
        assert_eq!(
            tree.root(),
            Fp::from_str_vartime(
                "15901536096620855161893017204769913722630450952860564330042326739189738305463"
            )
            .unwrap()
        );
    }
}
