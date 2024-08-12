use super::Blob;
use crate::types::Token;
use alloy::primitives::{Address, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key {
    Height,
    Block(usize),
    Delta(usize),
    ContractCode(Address),
    ContractStorage(Address, U256),
    TransactionHash(FixedBytes<32>),
    BlockHash(U256),
    TransactionCount,
    Transactions(Address),
    DepositedTransaction(String),
    Balance(Address, Token),
    Allowance(Address, Address, Token),
    NonceEth(Address),
    NonceCustom(Address),
    BurnId(FixedBytes<32>),
    TokenDecimal(Address),
    TokenSymbol(Address),
}

impl TryInto<Blob> for Key {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<Blob> {
        Ok(Blob(bincode::serialize(&self)?))
    }
}

impl TryInto<Blob> for &Key {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<Blob> {
        Ok(Blob(bincode::serialize(&self)?))
    }
}
