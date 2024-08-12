use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{Key, KvStore, MirrorKvStore, Value};
use crate::services::ContextKvStore;
use crate::types::{
    BincodableOwshenTransaction, Block, CustomTxMsg, IncludedTransaction, OwshenTransaction, Token,
};

use alloy::primitives::{Address, FixedBytes, U256};
use anyhow::{anyhow, Result};

mod config;
pub use config::Config;
mod ovm;
pub mod tx;

pub trait Blockchain {
    fn config(&self) -> &Config;
    fn get_block(&self, index: usize) -> Result<Block>;
    fn get_last_block(&self) -> Result<Option<Block>>;
    fn get_height(&self) -> Result<usize>;
    fn get_balance(&self, token: Token, address: Address) -> Result<U256>;
    fn get_allowance(&self, owner: Address, spender: Address, token: Token) -> Result<U256>;
    fn get_custom_nonce(&self, address: Address) -> Result<U256>;
    fn get_eth_nonce(&self, address: Address) -> Result<U256>;
    fn push_block(&mut self, block: Block) -> Result<()>;
    fn pop_block(&mut self) -> Result<Option<Block>>;
    fn draft_block(&self, txs: &mut TransactionQueue, timestamp: u64) -> Result<Block>;
    fn get_transactions_by_block(&self, block_index: usize) -> Result<Vec<OwshenTransaction>>;
    fn get_transaction_by_hash(
        &self,
        tx_hash: FixedBytes<32>,
    ) -> Result<IncludedTransaction, anyhow::Error>;
    fn get_user_withdrawals(&self, address: Address) -> Result<Vec<IncludedTransaction>>;
    fn get_total_transactions(&self) -> Result<U256>;
    fn get_transactions_per_second(&self) -> Result<f64>;
    fn get_transactions_by_block_paginated(
        &self,
        index: usize,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<OwshenTransaction>>;
    fn get_blocks(&self, offset: usize, limit: usize) -> Result<Vec<Block>>;
    fn get_token_decimal(&self, token_address: Address) -> Result<U256>;
    fn get_token_symbol(&self, token_address: Address) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct Owshenchain<K: ContextKvStore> {
    config: Config,
    pub db: K,
}
#[derive(Debug, Clone)]
pub struct TransactionQueue {
    queue: VecDeque<OwshenTransaction>,
}

impl TransactionQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, tx: OwshenTransaction) {
        self.queue.push_back(tx);
    }

    pub fn dequeue(&mut self) -> Option<OwshenTransaction> {
        self.queue.pop_front()
    }

    pub fn queue(&self) -> &VecDeque<OwshenTransaction> {
        &self.queue
    }
}

impl<K: ContextKvStore> Owshenchain<K> {
    pub fn new(config: Config, db: K) -> Self {
        Self { config, db }
    }
    fn fork<'a>(&'a self) -> Owshenchain<MirrorKvStore<'a, K>> {
        Owshenchain {
            config: self.config.clone(),
            db: MirrorKvStore::new(&self.db),
        }
    }

    fn atomic<R, F: FnOnce(&mut Owshenchain<MirrorKvStore<'_, K>>) -> Result<R>>(
        &mut self,
        f: F,
    ) -> Result<R> {
        let mut fork = self.fork();
        let ret = (f)(&mut fork)?;
        let buff = fork.db.buffer();
        self.db.batch_put_raw(buff.clone().into_iter())?;
        Ok(ret)
    }

    fn apply_tx(&mut self, tx: &OwshenTransaction) -> Result<()> {
        self.atomic(|chain| {
            let from = tx.signer()?;

            if tx.chain_id()? != chain.config.chain_id {
                return Err(anyhow!("Chain id is not valid!"));
            }

            match tx {
                OwshenTransaction::Custom(custom_tx) => match custom_tx.msg()? {
                    // CustomTxMsg::OwshenAirdrop {
                    //     owshen_address,
                    //     owshen_sig,
                    // } => {
                    //     log::info!("Someone is claiming his owshen airdrop, by {}!", from);
                    // }
                    CustomTxMsg::MintTx(mint_data) => {
                        tx::mint_tx(
                            chain,
                            mint_data.tx_hash.to_vec(),
                            mint_data.user_tx_hash,
                            mint_data.token,
                            mint_data.amount,
                            mint_data.address,
                        )?;
                        log::info!("Mint transaction, by {}!", from);
                    }
                    CustomTxMsg::BurnTx(burn_data) => {
                        tx::burn_tx(chain, burn_data)?;
                        log::info!("Burn transaction, by {}!", from);
                    }
                },
                OwshenTransaction::Eth(eth_tx) => {
                    tx::eth(chain, from, eth_tx)?;
                }
            }
            Ok(())
        })
    }

    fn store_block_hash(&mut self, block: Block) -> Result<()> {
        let block_hash = block.hash()?;
        let key = Key::BlockHash(U256::from_be_bytes(block_hash.into()));
        let value = Value::Block(block);
        self.db.put(key, Some(value))
    }
}

impl<K: ContextKvStore> Blockchain for Owshenchain<K> {
    fn config(&self) -> &Config {
        &self.config
    }
    fn get_block(&self, index: usize) -> Result<Block> {
        if index >= self.get_height()? {
            Err(anyhow!("Block doesn't exist!"))
        } else if let Some(blk) = self.db.get(Key::Block(index))? {
            blk.as_block()
        } else {
            Err(anyhow!("Inconsistency!"))
        }
    }
    fn get_last_block(&self) -> Result<Option<Block>> {
        let height = self.get_height()?;
        if height > 0 {
            Ok(Some(self.get_block(height - 1)?))
        } else {
            Ok(None)
        }
    }
    fn get_height(&self) -> Result<usize> {
        if let Some(v) = self.db.get(Key::Height)? {
            v.as_usize()
        } else {
            Ok(0)
        }
    }
    fn get_custom_nonce(&self, address: Address) -> Result<U256> {
        if let Some(v) = self.db.get(Key::NonceCustom(address))? {
            v.as_u256()
        } else {
            Ok(U256::from(0))
        }
    }
    fn get_eth_nonce(&self, address: Address) -> Result<U256> {
        if let Some(v) = self.db.get(Key::NonceEth(address))? {
            v.as_u256()
        } else {
            Ok(U256::from(0))
        }
    }

    fn get_balance(&self, token: Token, address: Address) -> Result<U256> {
        if let Some(v) = self.db.get(Key::Balance(address, token.clone()))? {
            return v.as_u256();
        }
        if let Some(genesis_balance) = self
            .config
            .genesis
            .tokens
            .get(&token)
            .and_then(|balances| balances.get(&address))
        {
            return Ok(*genesis_balance);
        }
        Ok(U256::from(0))
    }

    fn get_allowance(&self, owner: Address, spender: Address, token: Token) -> Result<U256> {
        if let Some(v) = self.db.get(Key::Allowance(owner, spender, token.clone()))? {
            return v.as_u256();
        } else {
            Ok(U256::from(0))
        }
    }

    fn push_block(&mut self, block: Block) -> Result<()> {
        self.atomic(move |chain| {
            let height = chain.get_height()?;
            let curr_hash = if let Some(b) = chain.get_last_block()? {
                Some(b.hash()?)
            } else {
                None
            };

            if block.index != height {
                return Err(anyhow!("Bad previous hash!"));
            }

            if block.prev_hash != curr_hash {
                return Err(anyhow!("Bad block index!"));
            }

            if let Some(owner) = chain.config().owner {
                if !block.is_signed_by(owner)? {
                    return Err(anyhow!("Block is not correctly signed!"));
                }
            }

            let tx_count = chain.get_total_transactions()?;
            let new_tx_count = tx_count + U256::from(block.txs.len());
            chain
                .db
                .put(Key::TransactionCount, Some(Value::U256(new_tx_count)))?;

            for (ind, bin_tx) in block.txs.iter().enumerate() {
                let tx = bin_tx.try_into()?;
                chain.apply_tx(&tx)?;
                chain.db.put(
                    Key::TransactionHash(tx.hash()?),
                    Some(Value::Transaction(IncludedTransaction {
                        transaction_index: ind,
                        tx: bin_tx.clone(),
                        block_hash: block.hash()?,
                        block_number: block.index,
                    })),
                )?;

                // add tx to user_transactions
                let key = Key::Transactions(tx.signer()?);
                let mut transactions: Vec<IncludedTransaction> = match chain.db.get(key.clone())? {
                    Some(Value::Transactions(existing_transactions)) => existing_transactions,
                    None => Vec::new(),
                    _ => return Err(anyhow!("Unexpected value type")),
                };

                let included_transaction = IncludedTransaction {
                    tx: bin_tx.clone(),
                    block_hash: block.hash()?,
                    block_number: block.index,
                    transaction_index: ind,
                };

                transactions.push(included_transaction);

                chain.db.put(key, Some(Value::Transactions(transactions)))?;
            }

            let _ = chain.store_block_hash(block.clone());
            chain.db.put(Key::Height, Some(Value::Usize(height + 1)))?;
            chain
                .db
                .put(Key::Block(height), Some(Value::Block(block)))?;

            let delta = chain.db.rollback()?;
            chain
                .db
                .put(Key::Delta(height + 1), Some(Value::BTreeMap(delta.clone())))?;
            Ok(())
        })
    }
    fn pop_block(&mut self) -> Result<Option<Block>> {
        self.atomic(|chain| {
            let height = chain.get_height()?;
            if height == 0 {
                Ok(None)
            } else {
                let block = chain.get_block(height - 1)?;
                let tx_count = chain.get_total_transactions()?;
                let new_tx_count = tx_count + U256::from(block.txs.len());
                chain
                    .db
                    .put(Key::TransactionCount, Some(Value::U256(new_tx_count)))?;

                let delta_blob = chain
                    .db
                    .get(Key::Delta(height))?
                    .ok_or(anyhow!("Delta not found!"))?;
                chain
                    .db
                    .batch_put_raw(delta_blob.as_btreemap()?.into_iter())?;
                chain.db.put(Key::Delta(height), None)?;
                Ok(Some(block))
            }
        })
    }
    fn draft_block(&self, txs: &mut TransactionQueue, timestamp: u64) -> Result<Block> {
        let mut selected_txs = Vec::new();
        let mut fork = self.fork();

        while let Some(tx) = txs.dequeue() {
            if fork.apply_tx(&tx).is_ok() {
                selected_txs.push(tx)
            } else {
                //pass
            }
        }

        let prev_hash = if let Some(b) = self.get_last_block()? {
            Some(b.hash()?)
        } else {
            None
        };

        let blk = Block {
            prev_hash,
            index: self.get_height()?,
            txs: selected_txs
                .iter()
                .map(|t| t.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            sig: None,
            timestamp,
        };

        Ok(blk)
    }

    fn get_transactions_by_block(&self, index: usize) -> Result<Vec<OwshenTransaction>> {
        let blk = self.get_block(index)?;
        let transactions = blk
            .txs
            .into_iter()
            .map(|t| (&t).try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(transactions)
    }

    fn get_transactions_by_block_paginated(
        &self,
        index: usize,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<OwshenTransaction>> {
        let blk = self.get_block(index)?;
        let total_txs = blk.txs.len();
        if offset >= total_txs {
            return Ok(vec![]);
        }
        let end = std::cmp::min(offset + limit, total_txs);

        let paginated_txs = blk.txs[offset..end]
            .iter()
            .map(|t| t.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paginated_txs)
    }

    fn get_transaction_by_hash(&self, tx_hash: FixedBytes<32>) -> Result<IncludedTransaction> {
        let key = Key::TransactionHash(tx_hash);
        if let Some(Value::Transaction(tx)) = self.db.get(key)? {
            Ok(tx.try_into()?)
        } else {
            Err(anyhow!("Transaction not found!"))
        }
    }

    fn get_user_withdrawals(&self, address: Address) -> Result<Vec<IncludedTransaction>> {
        let key = Key::Transactions(address);

        if let Some(Value::Transactions(included_transactions)) = self.db.get(key)? {
            let withdrawals: Vec<IncludedTransaction> = included_transactions
                .into_iter()
                .filter(|included_tx| {
                    if let Ok(OwshenTransaction::Custom(custom_tx)) =
                        included_tx.tx.clone().try_into()
                    {
                        if let Ok(CustomTxMsg::BurnTx(_)) = custom_tx.msg() {
                            return true;
                        }
                    }
                    false
                })
                .collect();

            Ok(withdrawals)
        } else {
            Err(anyhow!("No transactions found for this user!"))
        }
    }

    fn get_total_transactions(&self) -> Result<U256> {
        if let Some(v) = self.db.get(Key::TransactionCount)? {
            v.as_u256()
        } else {
            Ok(U256::default())
        }
    }

    fn get_transactions_per_second(&self) -> Result<f64> {
        let height = self.get_height()?;

        if height < 2 {
            return Ok(0.0);
        }

        let last_block = self.get_block(height - 1)?;
        let previous_block = self.get_block(height - 2)?;
        let last_timestamp = last_block.timestamp as f64;
        let previous_timestamp = previous_block.timestamp as f64;
        let num_transactions = last_block.txs.len() as f64;
        let time_span = last_timestamp - previous_timestamp;

        if time_span > 0.0 {
            let tps = num_transactions / time_span;
            return Ok(tps);
        }

        Ok(0.0)
    }

    fn get_blocks(&self, offset: usize, limit: usize) -> Result<Vec<Block>> {
        let height = self.get_height()?;

        if offset >= height {
            return Ok(Vec::new()); // No blocks to return
        }

        let end = std::cmp::min(offset + limit, height);

        let mut blocks = Vec::with_capacity(end - offset);
        for index in offset..end {
            if let Some(block) = self.get_block(index).ok() {
                blocks.push(block);
            } else {
                return Err(anyhow!("Inconsistency detected while fetching blocks"));
            }
        }

        Ok(blocks)
    }

    fn get_token_decimal(&self, token_address: Address) -> Result<U256> {
        if let Some(v) = self.db.get(Key::TokenDecimal(token_address))? {
            v.as_u256()
        } else {
            Ok(U256::from(0))
        }
    }
    fn get_token_symbol(&self, token_address: Address) -> Result<String> {
        if let Some(v) = self.db.get(Key::TokenSymbol(token_address))? {
            v.as_string()
        } else {
            Ok(String::from("Unknown"))
        }
    }
}

#[cfg(test)]
mod tests;
