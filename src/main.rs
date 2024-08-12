mod blockchain;
mod cli;
mod client;
mod config;
mod db;
mod genesis;
mod safe_signer;
mod services;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    cli::cli().await.unwrap();
}
