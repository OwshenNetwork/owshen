use alloy::signers::Signer;

use crate::{
    blockchain::{Owshenchain, TransactionQueue},
    db::KvStore,
};

mod api_services;
mod rpc_services;
pub mod server;

pub trait ContextSigner: Signer + Send + Sync + Clone {}

pub trait ContextKvStore: KvStore + Send + Sync {}

pub struct Context<S: ContextSigner, K: ContextKvStore> {
    pub exit: bool,
    pub signer: S,
    pub tx_queue: TransactionQueue,
    pub chain: Owshenchain<K>,
}
