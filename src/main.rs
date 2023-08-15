mod fp;
mod hash;
mod keys;
mod proof;



use keys::{PrivateKey, PublicKey};
use proof::{prove};



use bindings::counter::Counter;


use ethers::prelude::*;
use ethers::utils::Ganache;


use eyre::Result;


use std::sync::Arc;

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
