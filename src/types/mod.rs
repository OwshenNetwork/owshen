pub mod network;
mod tx;
use alloy::{
    primitives::{keccak256, Address, FixedBytes, U256},
    signers::Signer,
};
use anyhow::Result;
use network::Network;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
pub use tx::{
    BincodableOwshenTransaction, Burn, CustomTx, CustomTxMsg, IncludedTransaction, Mint,
    OwshenTransaction, WithdrawCalldata,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Block {
    pub prev_hash: Option<FixedBytes<32>>,
    pub index: usize,
    pub txs: Vec<BincodableOwshenTransaction>,
    pub sig: Option<alloy::primitives::Signature>,
    pub timestamp: u64,
}

impl Block {
    pub fn unsigned_bytes(&self) -> Result<Vec<u8>> {
        let mut blk = self.clone();
        blk.sig = None;
        Ok(bincode::serialize(&blk)?)
    }
    pub fn hash(&self) -> Result<FixedBytes<32>> {
        Ok(keccak256(self.unsigned_bytes()?))
    }
    pub async fn signed<S: Signer + Sync>(&self, signer: S) -> Result<Block> {
        let bytes = self.hash()?;
        let mut blk = self.clone();

        blk.sig = Some(signer.sign_hash(&bytes).await?);
        Ok(blk)
    }
    pub fn is_signed_by(&self, addr: Address) -> Result<bool> {
        let hash = self.hash()?;
        Ok(if let Some(sig) = self.sig {
            sig.recover_address_from_prehash(&hash)? == addr
        } else {
            false
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Deserialize)]
pub enum Token {
    Native,
    Erc20(ERC20),
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Deserialize)]
pub struct ERC20 {
    pub address: Address,
    pub decimals: U256,
    pub symbol: String,
}
