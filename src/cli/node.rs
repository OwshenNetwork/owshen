use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::{
    primitives::U256,
    signers::{local::PrivateKeySigner, Signer},
};
use anyhow::Result;
use tokio::sync::Mutex;

use crate::{
    blockchain::{Blockchain, Config, Owshenchain, TransactionQueue},
    config,
    db::KvStore,
    genesis::GENESIS,
    safe_signer::SafeSigner,
    services::{
        server::{api_server, rpc_server},
        Context, ContextKvStore, ContextSigner,
    },
    types::{CustomTx, CustomTxMsg, Mint, Token},
};

async fn block_producer<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Mutex<Context<S, K>>>,
) -> Result<()> {
    loop {
        if let Err(e) = async {
            let mut ctx = ctx.lock().await;
            if ctx.exit {
                log::info!("Terminating the block producer...");
                return Ok(());
            }

            let mut tx_queue = std::mem::replace(&mut ctx.tx_queue, TransactionQueue::new());
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let mut blk = ctx.chain.draft_block(&mut tx_queue, timestamp)?;
            ctx.tx_queue = tx_queue;

            blk = blk.signed(ctx.signer.clone()).await?;
            ctx.chain.push_block(blk.clone())?;
            log::info!("Produced a new block: {}", blk.index);
            Ok::<(), anyhow::Error>(())
        }
        .await
        {
            log::info!("Error while producing a block: {}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

pub async fn run_node<K: ContextKvStore + 'static>(
    db: K,
    api_port: u16,
    rpc_port: u16,
    provider_address: reqwest::Url,
    private_key: PrivateKeySigner,
) -> Result<()> {
    let signer = SafeSigner::new(private_key);
    let conf = Config {
        chain_id: config::CHAIN_ID,
        owner: Some(signer.address()),
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address,
    };

    let ctx = Arc::new(Mutex::new(Context {
        signer: signer.clone(),
        exit: false,
        tx_queue: TransactionQueue::new(),
        chain: Owshenchain::new(conf.clone(), db),
    }));

    let tx = CustomTx::create(
        &mut signer.clone(),
        conf.chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: vec![0u8; 32],
            user_tx_hash: "0x1234567890abcdef".to_string(),
            token: Token::Native,
            amount: U256::from(100),
            address: PrivateKeySigner::random().address(),
        }),
    )
    .await?;
    ctx.lock().await.tx_queue.enqueue(tx);

    let block_producer_fut = block_producer(ctx.clone());
    let api_server_fut = api_server(ctx.clone(), api_port);
    let rpc_server_fut = rpc_server(ctx.clone(), rpc_port);

    let entrypoint = format!("http://127.0.0.1:{}", api_port);
    if webbrowser::open(&entrypoint).is_err() {
        println!("Failed to open web browser. Please navigate to http://{entrypoint} manually");
    }

    tokio::try_join!(block_producer_fut, api_server_fut, rpc_server_fut)?;

    Ok(())
}
