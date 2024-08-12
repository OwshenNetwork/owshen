use alloy::{
    primitives::{Address, ChainId, Signature, B256},
    signers::{local::PrivateKeySigner, Signer},
};
use async_trait::async_trait;

use crate::services::ContextSigner;

#[derive(Clone)]
pub struct SafeSigner {
    private_key: PrivateKeySigner,
}

impl SafeSigner {
    pub fn new(private_key: PrivateKeySigner) -> Self {
        Self { private_key }
    }
}

impl ContextSigner for SafeSigner {}

#[async_trait]
impl Signer for SafeSigner {
    async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        self.private_key.sign_hash(hash).await
    }

    fn address(&self) -> Address {
        self.private_key.address()
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.private_key.chain_id()
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.private_key.set_chain_id(chain_id)
    }
}
