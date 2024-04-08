use crate::keys::PublicKey;
use crate::{fmt::FMTProof, fp::Fp};

use ethers::{abi::ethabi::ethereum_types::FromStrRadixErr, prelude::*};
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

pub fn prove<P: AsRef<Path>>(
    token_address: U256,

    index: Vec<u32>,
    amount: Vec<U256>,
    secret: Vec<Fp>,

    proof: Vec<FMTProof>,

    new_amount: Vec<U256>,
    pk: Vec<PublicKey>,

    params: P,
    witness_gen_path: String,
    prover_path: String,
) -> Result<Proof> {
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
        serde_json::to_string(&index).ok().unwrap(),
        BigUint::from_str(&token_address.to_string()).ok().unwrap(),
        serde_json::to_string(&amount).ok().unwrap(),
        serde_json::to_string(&secret).ok().unwrap(),
        serde_json::to_string(&proof[0].checkpoint_head)
            .ok()
            .unwrap(),
        serde_json::to_string(&proof[0].latest_values_commitment_head)
            .ok()
            .unwrap(),
        serde_json::to_string(&proof.iter().map(|p| p.value).collect::<Vec<Fp>>())
            .ok()
            .unwrap(),
        serde_json::to_string(
            &proof
                .iter()
                .map(|p| p.between_values.clone())
                .collect::<Vec<Vec<Fp>>>()
        )
        .ok()
        .unwrap(),
        serde_json::to_string(&proof[0].checkpoint_commitments.clone())
            .ok()
            .unwrap(),
        serde_json::to_string(&proof[0].checkpoints.clone())
            .ok()
            .unwrap(),
        serde_json::to_string(&proof[0].latest_values.clone())
            .ok()
            .unwrap(),
        serde_json::to_string(
            &proof
                .iter()
                .map(|p| if p.is_in_latest_commits { 1 } else { 0 })
                .collect::<Vec<u64>>()
        )
        .ok()
        .unwrap(),
        serde_json::to_string(&new_amount).ok().unwrap(),
        serde_json::to_string(&pk.iter().map(|pk| pk.point.x).collect::<Vec<Fp>>())
            .ok()
            .unwrap(),
        serde_json::to_string(&pk.iter().map(|pk| pk.point.y).collect::<Vec<Fp>>())
            .ok()
            .unwrap()
    );

    write!(inputs_file, "{}", json_input)?;

    log::info!("Circuit input: {}", json_input);

    let witness_file = NamedTempFile::new()?;
    let wtns_gen_output = Command::new(witness_gen_path)
        .arg(inputs_file.path())
        .arg(witness_file.path())
        .output()?;

    if !wtns_gen_output.stdout.is_empty() {
        log::info!(
            "Witness generator output: {}",
            String::from_utf8_lossy(&wtns_gen_output.stdout)
        );
    }
    if !wtns_gen_output.stderr.is_empty() {
        log::error!(
            "Error while generating witnesses: {}",
            String::from_utf8_lossy(&wtns_gen_output.stderr)
        );
    }

    assert_eq!(wtns_gen_output.stdout.len(), 0);
    assert_eq!(wtns_gen_output.stderr.len(), 0);

    let proof_file = NamedTempFile::new()?;
    let pub_inp_file = NamedTempFile::new()?;
    let proof_gen_output = Command::new(prover_path)
        .arg(params.as_ref().as_os_str())
        .arg(witness_file.path())
        .arg(proof_file.path())
        .arg(pub_inp_file.path())
        .output()?;

    if !proof_gen_output.stdout.is_empty() {
        log::info!(
            "Proof generator output: {}",
            String::from_utf8_lossy(&proof_gen_output.stdout)
        );
    }

    if !proof_gen_output.stderr.is_empty() {
        log::error!(
            "Error while generating proof: {}",
            String::from_utf8_lossy(&proof_gen_output.stderr)
        );
    }

    assert_eq!(proof_gen_output.stdout.len(), 0);
    assert_eq!(proof_gen_output.stderr.len(), 0);

    let generatecall_output = Command::new("snarkjs")
        .arg("generatecall")
        .arg(pub_inp_file.path())
        .arg(proof_file.path())
        .output()?;

    let mut calldata = std::str::from_utf8(&generatecall_output.stdout)?.to_string();
    calldata = calldata
        .replace("\"", "")
        .replace("[", "")
        .replace("]", "")
        .replace(" ", "")
        .replace("\n", "");

    let data = calldata
        .split(",")
        .map(|k| U256::from_str_radix(k, 16))
        .collect::<Result<Vec<U256>, FromStrRadixErr>>()?;

    let proof = Proof {
        a: data[0..2].try_into()?,
        b: [data[2..4].try_into()?, data[4..6].try_into()?],
        c: data[6..8].try_into()?,
        public: data[8..].to_vec(),
    };

    Ok(proof)
}
