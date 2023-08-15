mod fp;
pub use fp::Fp;

use ff::{Field, PrimeField};

use bindings::counter::Counter;
use num_bigint::BigUint;

use ethers::prelude::*;
use ethers::utils::Ganache;

use ethers::abi::ethabi::ethereum_types::FromStrRadixErr;
use eyre::Result;
use rand::Rng;
use std::process::Command;
use std::sync::Arc;

fn hash(left: Fp, right: Fp) -> Fp {
    left * right // Dummy hash function!
}

#[derive(Debug, Clone)]
struct PrivateKey {
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
struct PublicKey {
    commitment: Fp,
}

impl From<PrivateKey> for PublicKey {
    fn from(sk: PrivateKey) -> Self {
        Self {
            commitment: hash(sk.secret, Fp::from(1)),
        }
    }
}

#[derive(Clone, Debug)]
struct Proof {
    a: [U256; 2],
    b: [[U256; 2]; 2],
    c: [U256; 2],
    public: Vec<U256>,
}

use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

fn prove<P: AsRef<Path>>(params: P, a: Fp, b: Fp) -> Result<Proof> {
    let mut inputs_file = NamedTempFile::new()?;
    write!(
        inputs_file,
        "{{ \"a\": {:?}, \"b\": {:?} }}",
        BigUint::from_bytes_le(a.to_repr().as_ref()),
        BigUint::from_bytes_le(b.to_repr().as_ref())
    )?;

    let mut witness_file = NamedTempFile::new()?;
    let wtns_gen_output = Command::new("contracts/circuits/coin_withdraw_cpp/coin_withdraw")
        .arg(inputs_file.path())
        .arg(witness_file.path())
        .output()?;

    assert_eq!(wtns_gen_output.stdout.len(), 0);
    assert_eq!(wtns_gen_output.stderr.len(), 0);

    let mut proof_file = NamedTempFile::new()?;
    let mut pub_inp_file = NamedTempFile::new()?;
    let proof_gen_output = Command::new("snarkjs")
        .arg("groth16")
        .arg("prove")
        .arg(params.as_ref().as_os_str())
        .arg(witness_file.path())
        .arg(proof_file.path())
        .arg(pub_inp_file.path())
        .output()?;

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Welcome to Owshen Client!");

    let sk = PrivateKey::generate(&mut rand::thread_rng());
    println!("Public key: {:?}", PublicKey::from(sk.clone()));

    let params_file = "contracts/circuits/coin_withdraw_0001.zkey";

    println!("Proof: {:?}", prove(params_file, 123.into(), 234.into())?);

    let port = 8545u16;
    let url = format!("http://localhost:{}", port).to_string();

    let ganache = Ganache::new()
        .port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    let provider = Provider::<Http>::try_from(url)?;
    let provider = Arc::new(provider);

    let accounts = provider.get_accounts().await?;
    let from = accounts[0];

    let counter = Counter::deploy(provider.clone(), ())?
        .legacy()
        .from(from)
        .send()
        .await?;

    counter
        .set_number(1234.into())
        .legacy()
        .from(from)
        .send()
        .await?;

    let num_req = counter.number().legacy().from(from);
    let num = num_req.call().await?;

    println!("{:?}", num);

    drop(ganache);

    Ok(())
}
