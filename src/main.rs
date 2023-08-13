mod fp;

use bindings::counter::Counter;

use ethers::prelude::*;

use eyre::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    /*let port = 8545u16;
    let url = format!("http://localhost:{}", port).to_string();

    let ganache = Ganache::new()
        .port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();*/
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;
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

    Ok(())
}
