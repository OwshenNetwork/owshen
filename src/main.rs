mod fp;
use fp::Fp;

use bindings::counter::Counter;

use ethers::{prelude::Middleware, providers::test_provider::GOERLI, types::Address};

use eyre::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let provider = GOERLI.provider();
    let provider = Arc::new(provider);

    let counter = bindings::counter::Counter::deploy(Arc::clone(&provider), ())?.send().await?;

    let addr = counter.address();
    println!("Counter.sol has been deployed to {:?}", addr);

    let address = "0x0000000000000000000000000000000000000000".parse::<Address>()?;

    let contract = Counter::new(address, provider);
    let blk = contract.client().get_block_number().await?;
    println!("Hello, world! {}", blk);
    Ok(())
}