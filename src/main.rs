mod fp;
pub use fp::Fp;

use ff::Field;

use bindings::counter::Counter;

use ethers::prelude::*;
use ethers::utils::Ganache;

use eyre::Result;
use rand::Rng;
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Welcome to Owshen Client!");

    let sk = PrivateKey::generate(&mut rand::thread_rng());
    println!("Public key: {:?}", PublicKey::from(sk.clone()));

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
