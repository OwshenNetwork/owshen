use super::Blob;
use crate::types::{BincodableOwshenTransaction, Block, IncludedTransaction, OwshenTransaction};
use alloy::primitives::U256;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Void,
    Usize(usize),
    U256(U256),
    BTreeMap(BTreeMap<Blob, Option<Blob>>),
    Block(Block),
    VecU8(Vec<u8>),
    Transaction(IncludedTransaction),
    Transactions(Vec<IncludedTransaction>),
    DepositedTransaction(String),
    Symbol(String)

}

impl TryInto<Blob> for Value {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<Blob> {
        (&self).try_into()
    }
}

impl TryInto<Blob> for &Value {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<Blob> {
        Ok(Blob(bincode::serialize(&self)?))
    }
}

impl Value {
    pub fn as_usize(&self) -> Result<usize> {
        match self {
            Value::Usize(v) => Ok(*v),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }

    pub fn as_u256(&self) -> Result<U256> {
        match self {
            Value::U256(n) => Ok(*n),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self {
            Value::Symbol(n) => Ok(n.clone()),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }

    pub fn as_btreemap(&self) -> Result<BTreeMap<Blob, Option<Blob>>> {
        match self {
            Value::BTreeMap(v) => Ok(v.clone()),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }
    pub fn as_block(&self) -> Result<Block> {
        match self {
            Value::Block(v) => Ok(v.clone()),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }
    pub fn as_vec_u8(&self) -> Result<Vec<u8>> {
        match self {
            Value::VecU8(v) => Ok(v.clone()),
            _ => Err(anyhow!("Unexpected type!")),
        }
    }
}
