use crate::fp::Fp;
use crate::hash::hash4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMerkleTree {
    defaults: Vec<Fp>,
    layers: Vec<HashMap<u64, Fp>>,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub value: Fp,
    pub proof: Vec<[Fp; 3]>,
}

impl SparseMerkleTree {
    pub fn depth(&self) -> usize {
        self.layers.len() - 1
    }

    pub fn new(depth: usize) -> Self {
        let mut defaults = vec![Fp::from(0)];
        for i in 0..depth {
            defaults.push(hash4([defaults[i], defaults[i], defaults[i], defaults[i]]));
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
        if self.get_at_layer(0, index) == value {
            return; // Already set! (Optimization)
        }
        for layer in 0..self.depth() + 1 {
            self.layers[layer].insert(index, value);

            let leftmost_leaf = index - (index % 4);
            let mut vals = (0..4)
                .map(|i| self.get_at_layer(layer, leftmost_leaf + i as u64))
                .collect::<Vec<_>>();
            vals[(index % 4) as usize] = value;
            value = hash4(vals.try_into().unwrap());
            index /= 4;
        }
    }

    pub fn get(&self, mut index: u64) -> MerkleProof {
        let value = self.get_at_layer(0, index);
        let mut proof = vec![];
        for layer in 0..self.depth() {
            let leftmost_leaf = index - (index % 4);
            let mut vals = (0..4)
                .map(|i| self.get_at_layer(layer, leftmost_leaf + i as u64))
                .collect::<Vec<_>>();
            vals.remove((index % 4) as usize);
            proof.push(vals.try_into().unwrap());
            index /= 4;
        }
        MerkleProof { value, proof }
    }

    pub fn root(&self) -> Fp {
        self.get_at_layer(self.depth(), 0)
    }

    pub fn genesis_root(&self) -> Fp {
        self.get_at_layer(self.depth() - 1, 0)
    }

    #[allow(dead_code)]
    pub fn verify(root: Fp, mut index: u64, proof: &MerkleProof) -> bool {
        let mut value = proof.value;
        for p in proof.proof.iter() {
            let mut vals = p.to_vec();
            vals.insert((index % 4) as usize, value);
            value = hash4(vals.try_into().unwrap());
            index /= 4;
        }
        value == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_trees() {
        let mut tree = SparseMerkleTree::new(16);
        tree.set(123, Fp::from(234));
        tree.set(345, Fp::from(456));
        let res = tree.get(123);
        let res2 = tree.get(345);
        let res3 = tree.get(200);
        assert!(SparseMerkleTree::verify(tree.root(), 123, &res));
        assert!(SparseMerkleTree::verify(tree.root(), 345, &res2));
        assert!(SparseMerkleTree::verify(tree.root(), 200, &res3));
        assert!(!SparseMerkleTree::verify(tree.root(), 123, &res2));
    }
}
