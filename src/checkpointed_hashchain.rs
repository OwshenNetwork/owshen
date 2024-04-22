use crate::fp::Fp;
use crate::hash::hash2;
use serde::{Deserialize, Serialize};

const CHECKPOINT_INTERVAL: u64 = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointedHashchain {
    values: Vec<Fp>,
    checkpoints: Vec<Fp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointedHashchainProof {
    pub checkpoint_head: Fp,
    pub latest_values_commitment_head: Fp,

    pub value: Fp,
    pub between_values: Vec<Fp>,

    pub checkpoint_commitments: Vec<Fp>,
    pub checkpoints: Vec<Fp>,

    pub latest_values: Vec<Fp>,
    pub is_in_latest_commits: bool,
}

impl CheckpointedHashchain {
    pub fn new() -> Self {
        Self {
            values: vec![],
            checkpoints: vec![],
        }
    }

    pub fn set(&mut self, value: Fp) {
        self.values.push(value);

        let index = self.values.len() as u64;

        if index % CHECKPOINT_INTERVAL == 0 && index > 0 {
            let mut commitment = self.values[(index - CHECKPOINT_INTERVAL) as usize];
            for i in 1..CHECKPOINT_INTERVAL {
                commitment = hash2([
                    commitment,
                    self.values[(index - CHECKPOINT_INTERVAL + i) as usize],
                ]);
            }
            if self.checkpoints.is_empty() {
                self.checkpoints.push(commitment);
            } else {
                let perv_checkpoint = self.checkpoints.last().cloned().unwrap_or_default();
                let checkpoint = hash2([perv_checkpoint, commitment]);
                self.checkpoints.push(checkpoint);
            }
        }
    }

    pub fn get(&self, index: u64) -> CheckpointedHashchainProof {
        let value = self.values[index as usize];

        let mut is_in_latest_commits = false;
        let mut between_values = vec![];
        if index < self.checkpoints.len() as u64 * CHECKPOINT_INTERVAL && self.checkpoints.len() > 0
        {
            between_values = (0..CHECKPOINT_INTERVAL)
                .map(|i| {
                    self.values[(index / CHECKPOINT_INTERVAL * CHECKPOINT_INTERVAL + i) as usize]
                })
                .collect();
        } else {
            for _ in 0..CHECKPOINT_INTERVAL {
                between_values.push(Fp::from(0));
            }
            is_in_latest_commits = true;
        }

        let mut checkpoint_commitments: Vec<Fp> = (0..self.checkpoints.len())
            .map(|i| {
                let mut commitment = self.values[i * CHECKPOINT_INTERVAL as usize];
                for j in 1..CHECKPOINT_INTERVAL {
                    commitment = hash2([
                        commitment,
                        self.values[i * CHECKPOINT_INTERVAL as usize + j as usize],
                    ]);
                }
                commitment
            })
            .collect();
        while checkpoint_commitments.len() < CHECKPOINT_INTERVAL as usize {
            checkpoint_commitments.push(Fp::from(0));
        }

        let mut checkpoints = self.checkpoints.clone();
        while checkpoints.len() < CHECKPOINT_INTERVAL as usize {
            checkpoints.push(Fp::from(0));
        }

        let mut latest_values = self.values
            [(self.values.len() as u64 / CHECKPOINT_INTERVAL * CHECKPOINT_INTERVAL) as usize..]
            .to_vec();
        let mut latest_values_commitment_head = latest_values.first().cloned().unwrap_or_default();
        for i in 1..latest_values.len() {
            latest_values_commitment_head =
                hash2([latest_values_commitment_head, latest_values[i]]);
        }

        while latest_values.len() < CHECKPOINT_INTERVAL as usize {
            latest_values.push(Fp::from(0));
        }

        CheckpointedHashchainProof {
            checkpoint_head: self.checkpoints.last().cloned().unwrap_or_default(),
            latest_values_commitment_head: latest_values_commitment_head,
            value,
            between_values,
            checkpoint_commitments,
            checkpoints,
            latest_values: latest_values,
            is_in_latest_commits,
        }
    }

    #[allow(dead_code)]
    pub fn verify(index: u64, proof: &CheckpointedHashchainProof) -> bool {
        assert_eq!(proof.checkpoints.len(), proof.checkpoint_commitments.len());

        for i in 0..proof.checkpoints.len() {
            let mut prev_checkpoint = Fp::from(0);
            if i > 0 {
                prev_checkpoint = proof.checkpoints[i - 1];
            }
            if hash2([prev_checkpoint, proof.checkpoint_commitments[i]]) != proof.checkpoints[i] {
                return false;
            }
        }

        let mut seen = false;
        let mut commitment = Fp::from(0);
        if proof.is_in_latest_commits {
            for i in 0..proof.latest_values.len() - 1 {
                commitment = hash2([commitment, proof.latest_values[i as usize]]);
                if proof.value == proof.latest_values[i as usize] {
                    seen = true;
                }
            }
        } else {
            for i in 0..proof.between_values.len() {
                commitment = hash2([commitment, proof.between_values[i as usize]]);
                if proof.value == proof.between_values[i as usize] {
                    seen = true;
                }
            }

            if proof.checkpoint_commitments[(index / CHECKPOINT_INTERVAL) as usize] != commitment {
                return false;
            }
        }

        if !seen {
            return false;
        }

        true
    }

    pub fn head(&self) -> Fp {
        self.values.last().cloned().unwrap_or_default()
    }

    pub fn get_last_checkpoint(&self) -> Fp {
        self.checkpoints.last().cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chc() {
        let mut chc = CheckpointedHashchain::new();
        chc.set(Fp::from(123));
        chc.set(Fp::from(234));
        chc.set(Fp::from(345));
        chc.set(Fp::from(456));
        chc.set(Fp::from(567));
        chc.set(Fp::from(678));
        chc.set(Fp::from(789));
        chc.set(Fp::from(890));
        chc.set(Fp::from(901));
        let proof = chc.get(2);
        assert!(CheckpointedHashchain::verify(1, &proof));

        let proof = chc.get(7);
        assert!(CheckpointedHashchain::verify(1, &proof));
    }
}
