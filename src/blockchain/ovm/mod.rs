use std::cell::RefCell;

use anyhow::Result;
use evm::{
    Capture, Context, CreateScheme, ExitError, ExitReason, ExternalOperation, Handler, Machine,
    Opcode, Stack, Transfer,
};
use primitive_types::{H160, H256, U256};

use crate::{
    db::{Key, KvStore, Value},
    services::ContextKvStore,
};

use super::Owshenchain;

// Owshen Virtual Machine
pub struct Ovm<'a, K: ContextKvStore> {
    error: RefCell<Option<anyhow::Error>>,
    chain: &'a mut Owshenchain<K>,
}

impl<'a, K: ContextKvStore> Ovm<'a, K> {
    pub fn new(chain: &'a mut Owshenchain<K>) -> Self {
        Self {
            chain,
            error: RefCell::new(None),
        }
    }
}

impl<'a, K: ContextKvStore> Handler for Ovm<'a, K> {
    type CallFeedback = ();
    type CallInterrupt = ();
    type CreateFeedback = ();
    type CreateInterrupt = ();
    fn balance(&self, _address: H160) -> U256 {
        unimplemented!()
    }
    fn block_base_fee_per_gas(&self) -> U256 {
        unimplemented!()
    }
    fn block_coinbase(&self) -> H160 {
        unimplemented!()
    }
    fn block_difficulty(&self) -> U256 {
        unimplemented!()
    }
    fn block_gas_limit(&self) -> U256 {
        unimplemented!()
    }
    fn block_hash(&self, _number: U256) -> H256 {
        unimplemented!()
    }
    fn block_number(&self) -> U256 {
        unimplemented!()
    }
    fn block_randomness(&self) -> Option<H256> {
        unimplemented!()
    }
    fn block_timestamp(&self) -> U256 {
        unimplemented!()
    }
    fn call(
        &mut self,
        _code_address: H160,
        _transfer: Option<Transfer>,
        _input: Vec<u8>,
        _target_gas: Option<u64>,
        _is_static: bool,
        _context: Context,
    ) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
        unimplemented!()
    }
    fn call_feedback(&mut self, _feedback: Self::CallFeedback) -> Result<(), ExitError> {
        unimplemented!()
    }
    fn chain_id(&self) -> U256 {
        unimplemented!()
    }
    fn code(&self, _address: H160) -> Vec<u8> {
        unimplemented!()
    }
    fn code_hash(&self, _address: H160) -> H256 {
        unimplemented!()
    }
    fn code_size(&self, _address: H160) -> U256 {
        unimplemented!()
    }
    fn create(
        &mut self,
        _caller: H160,
        _scheme: CreateScheme,
        _value: U256,
        _init_code: Vec<u8>,
        _target_gas: Option<u64>,
    ) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
        unimplemented!()
    }
    fn create_feedback(&mut self, _feedback: Self::CreateFeedback) -> Result<(), ExitError> {
        unimplemented!()
    }
    fn deleted(&self, _address: H160) -> bool {
        unimplemented!()
    }
    fn exists(&self, _address: H160) -> bool {
        unimplemented!()
    }
    fn gas_left(&self) -> U256 {
        unimplemented!()
    }
    fn gas_price(&self) -> U256 {
        unimplemented!()
    }
    fn is_cold(&mut self, _address: H160, _index: Option<H256>) -> Result<bool, ExitError> {
        unimplemented!()
    }
    fn log(&mut self, _address: H160, _topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
        println!("Log: {:?}", data);
        Ok(())
    }
    fn mark_delete(&mut self, _address: H160, _target: H160) -> Result<(), ExitError> {
        unimplemented!()
    }
    fn origin(&self) -> H160 {
        unimplemented!()
    }
    fn original_storage(&self, _address: H160, _index: H256) -> H256 {
        unimplemented!()
    }
    fn other(&mut self, _opcode: Opcode, _stack: &mut Machine) -> Result<(), ExitError> {
        unimplemented!()
    }
    fn pre_validate(
        &mut self,
        _context: &Context,
        _opcode: Opcode,
        _stack: &Stack,
    ) -> Result<(), ExitError> {
        if let Some(err) = self.error.borrow().as_ref() {
            return Err(ExitError::Other(format!("{}", err).into()));
        }
        Ok(())
    }
    fn record_external_operation(&mut self, _op: ExternalOperation) -> Result<(), ExitError> {
        unimplemented!()
    }
    fn set_storage(&mut self, address: H160, index: H256, _value: H256) -> Result<(), ExitError> {
        let index_alloy = alloy::primitives::U256::from_le_bytes(index.to_fixed_bytes());
        let address_alloy = alloy::primitives::Address::from_slice(&address.to_fixed_bytes());
        self.chain
            .db
            .put(
                Key::ContractStorage(address_alloy, index_alloy),
                Some(Value::U256(index_alloy)),
            )
            .map_err(|_| ExitError::Other("DB Failure".into()))?;
        Ok(())
    }
    fn storage(&self, address: H160, index: H256) -> H256 {
        let f = move || -> Result<H256> {
            let index_alloy = alloy::primitives::U256::from_le_bytes(index.to_fixed_bytes());
            let address_alloy = alloy::primitives::Address::from_slice(&address.to_fixed_bytes());
            Ok(
                if let Some(val) = self
                    .chain
                    .db
                    .get(Key::ContractStorage(address_alloy, index_alloy))?
                {
                    H256::from(val.as_u256()?.to_le_bytes())
                } else {
                    Default::default()
                },
            )
        };

        match f() {
            Ok(res) => res,
            Err(e) => {
                *self.error.borrow_mut() = Some(e);
                Default::default()
            }
        }
    }
}

#[cfg(test)]
mod tests;
