use crate::commands::wallet::Mode;
use crate::keys::PublicKey;
use crate::{checkpointed_hashchain::CheckpointedHashchainProof, fp::Fp};

use ethers::prelude::*;
use eyre::Result;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, process::Command, str::FromStr};
use tempfile::NamedTempFile;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Proof {
    pub a: [U256; 2],
    pub b: [[U256; 2]; 2],
    pub c: [U256; 2],
    pub public: Vec<U256>,
}

fn extract_proof(
    proof_obj: &serde_json::Value,
    pubs_obj: &serde_json::Value,
) -> Result<Proof, eyre::Report> {
    fn get(mut v: &serde_json::Value, k: &str, inds: &[usize]) -> Result<U256, eyre::Report> {
        v = v.get(k).ok_or(eyre::eyre!("Invalid proof object!"))?;
        for i in inds.iter() {
            v = v.get(i).ok_or(eyre::eyre!("Invalid proof object!"))?;
        }
        Ok(U256::from_str_radix(
            v.as_str().ok_or(eyre::eyre!("Invalid proof object!"))?,
            10,
        )?)
    }

    let proof_a = [
        get(&proof_obj, "pi_a", &[0])?,
        get(&proof_obj, "pi_a", &[1])?,
    ];
    let proof_b = [
        [
            get(&proof_obj, "pi_b", &[0, 0])?,
            get(&proof_obj, "pi_b", &[0, 1])?,
        ],
        [
            get(&proof_obj, "pi_b", &[1, 0])?,
            get(&proof_obj, "pi_b", &[1, 1])?,
        ],
    ];
    let proof_c = [
        get(&proof_obj, "pi_c", &[0])?,
        get(&proof_obj, "pi_c", &[1])?,
    ];

    let pubs = pubs_obj
        .as_array()
        .ok_or(eyre::eyre!("Invalid proof object!"))?
        .iter()
        .map(|v| v.as_str().ok_or(eyre::eyre!("Invalid proof object!")))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|v| U256::from_str_radix(v, 10))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Proof {
        a: proof_a,
        b: proof_b,
        c: proof_c,
        public: pubs,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProveResult {
    Proof(Proof),
    JsonInput(String),
}

pub fn prove<P: AsRef<Path>>(
    token_address: U256,

    index: Vec<u32>,
    amount: Vec<U256>,
    secret: Vec<Fp>,

    proof: Vec<CheckpointedHashchainProof>,

    new_amount: Vec<U256>,
    pk: Vec<PublicKey>,
    params: P,
    witness_gen_path: P,
    prover_path: P,
    mode: &Mode,
) -> Result<ProveResult> {
    let mut inputs_file = NamedTempFile::new()?;

    let json_input = format!(
        "{{ \"index\": {}, 
        \"token_address\": \"{}\", 
        \"amount\": {}, 
        \"secret\": {},
        
        \"user_checkpoint_head\": {},
        \"user_latest_values_commitment_head\": {},
        \"value\": {},
        \"between_values\": {},
        \"checkpoint_commitments\": {},
        \"checkpoints\": {},
        \"latest_values\": {},
        \"is_in_latest_commits\": {},
     
        \"new_amount\": {}, 
        \"pk_ax\": {}, 
        \"pk_ay\": {} }}",
        serde_json::to_string(&index)?,
        BigUint::from_str(&token_address.to_string())?,
        serde_json::to_string(&amount)?,
        serde_json::to_string(&secret)?,
        serde_json::to_string(&proof[0].checkpoint_head)?,
        serde_json::to_string(&proof[0].latest_values_commitment_head)?,
        serde_json::to_string(&proof.iter().map(|p| p.value).collect::<Vec<Fp>>())?,
        serde_json::to_string(
            &proof
                .iter()
                .map(|p| p.between_values.clone())
                .collect::<Vec<Vec<Fp>>>()
        )?,
        serde_json::to_string(&proof[0].checkpoint_commitments.clone())?,
        serde_json::to_string(&proof[0].checkpoints.clone())?,
        serde_json::to_string(&proof[0].latest_values.clone())?,
        serde_json::to_string(
            &proof
                .iter()
                .map(|p| if p.is_in_latest_commits { 1 } else { 0 })
                .collect::<Vec<u64>>()
        )?,
        serde_json::to_string(&new_amount)?,
        serde_json::to_string(&pk.iter().map(|pk| pk.point.x).collect::<Vec<Fp>>())?,
        serde_json::to_string(&pk.iter().map(|pk| pk.point.y).collect::<Vec<Fp>>())?
    );

    write!(inputs_file, "{}", json_input)?;

    log::info!("Circuit input: {}", json_input);

    if *mode == Mode::Windows {
        return Ok(ProveResult::JsonInput(json_input));
    }

    let witness_file = NamedTempFile::new()?;
    let wtns_gen_output = Command::new(witness_gen_path.as_ref().as_os_str())
        .arg(inputs_file.path())
        .arg(witness_file.path())
        .output()?;

    if !wtns_gen_output.stdout.is_empty() {
        return Err(eyre::eyre!(format!(
            "Error while generating witnesses: {}",
            String::from_utf8_lossy(&wtns_gen_output.stdout)
        )));
    }
    if !wtns_gen_output.stderr.is_empty() {
        return Err(eyre::eyre!(format!(
            "Error while generating witnesses: {}",
            String::from_utf8_lossy(&wtns_gen_output.stderr)
        )));
    }

    let proof_file = NamedTempFile::new()?;
    let pub_inp_file = NamedTempFile::new()?;
    let proof_gen_output = Command::new(prover_path.as_ref().as_os_str())
        .arg(params.as_ref().as_os_str())
        .arg(witness_file.path())
        .arg(proof_file.path())
        .arg(pub_inp_file.path())
        .output()?;

    if !proof_gen_output.stdout.is_empty() {
        return Err(eyre::eyre!(format!(
            "Error while generating proof: {}",
            String::from_utf8_lossy(&proof_gen_output.stdout)
        )));
    }
    if !proof_gen_output.stderr.is_empty() {
        return Err(eyre::eyre!(format!(
            "Error while generating proof: {}",
            String::from_utf8_lossy(&proof_gen_output.stderr)
        )));
    }

    let proof_obj: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(proof_file.path())?)?;
    let pubs_obj: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(pub_inp_file.path())?)?;

    Ok(ProveResult::Proof(extract_proof(&proof_obj, &pubs_obj)?))
}
