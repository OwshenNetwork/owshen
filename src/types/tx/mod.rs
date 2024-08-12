use alloy::rlp::{Decodable, Encodable};
use alloy::{
    consensus::{Transaction, TxEnvelope},
    network::{Ethereum, Network},
    primitives::{keccak256, Address, FixedBytes},
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
pub mod custom;
pub use custom::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BincodableOwshenTransaction {
    EncodedEth(Vec<u8>),
    Custom(CustomTx),
}

impl TryInto<OwshenTransaction> for &BincodableOwshenTransaction {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<OwshenTransaction> {
        match self {
            BincodableOwshenTransaction::EncodedEth(enc) => Ok(OwshenTransaction::Eth(
                <Ethereum as Network>::TxEnvelope::decode(&mut enc.as_ref())?,
            )),
            BincodableOwshenTransaction::Custom(tx) => Ok(OwshenTransaction::Custom(tx.clone())),
        }
    }
}
impl TryInto<OwshenTransaction> for BincodableOwshenTransaction {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<OwshenTransaction> {
        (&self).try_into()
    }
}

impl TryInto<BincodableOwshenTransaction> for &OwshenTransaction {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<BincodableOwshenTransaction> {
        match self {
            OwshenTransaction::Eth(tx) => {
                let mut buf = Vec::new();
                tx.encode(&mut buf);
                Ok(BincodableOwshenTransaction::EncodedEth(buf))
            }
            OwshenTransaction::Custom(tx) => Ok(BincodableOwshenTransaction::Custom(tx.clone())),
        }
    }
}
impl TryInto<BincodableOwshenTransaction> for OwshenTransaction {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<BincodableOwshenTransaction> {
        (&self).try_into()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Serialize, Clone, Deserialize)]
pub enum OwshenTransaction {
    Eth(<Ethereum as Network>::TxEnvelope),
    Custom(CustomTx),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludedTransaction {
    pub tx: BincodableOwshenTransaction,
    pub block_hash: FixedBytes<32>,
    pub block_number: usize,
    pub transaction_index: usize,
}

impl OwshenTransaction {
    pub fn chain_id(&self) -> Result<u64> {
        Ok(match self {
            Self::Eth(tx) => tx.chain_id().ok_or(anyhow!("Chain-id not provided!"))?,
            Self::Custom(tx) => tx.chain_id,
        })
    }
    pub fn signer(&self) -> Result<Address> {
        Ok(match self {
            Self::Eth(tx) => tx.recover_signer()?,
            Self::Custom(tx) => tx.signer()?,
        })
    }
    pub fn hash(&self) -> Result<FixedBytes<32>> {
        Ok(match self {
            Self::Eth(tx) => *tx.tx_hash(),
            Self::Custom(tx) => keccak256(bincode::serialize(&tx)?),
        })
    }
}

#[cfg(test)]
mod tests;
