use std::{ops::Add, rc::Rc};

use alloy::consensus::Transaction;
use alloy::primitives::Uint;
use alloy::rlp::bytes::buf::Chain;
use alloy::rlp::Decodable;
use alloy::{
    consensus::TxEnvelope,
    primitives::{keccak256, Address, TxKind, U256},
    sol_types::SolValue,
};
use alloy_sol_types::abi::token;
use anyhow::{anyhow, Error, Result};
use evm::{Capture, Context, ExitReason, Runtime};

use crate::types::{Token, ERC20};
use crate::{
    blockchain::{Blockchain, Owshenchain},
    db::{Key, KvStore, Value},
    services::ContextKvStore,
};

pub enum Erc20Operation {
    Transfer {
        receiver: Address,
        value: alloy::primitives::U256,
    },
    TransferFrom {
        from: Address,
        receiver: Address,
        value: alloy::primitives::U256,
    },
    Approve {
        spender: Address,
        value: alloy::primitives::U256,
    },
}

pub fn extract_erc20_transfer(tx: &TxEnvelope) -> Result<Option<Erc20Operation>> {
    let tx = tx
        .as_eip1559()
        .ok_or(anyhow!("Only EIP-1559 is supported!"))?;

    if tx.tx().value > U256::from(0) {
        return Ok(None);
    }

    if tx.tx().input.len() < 4 {
        return Err(anyhow!("Unknown transaction input!"));
    }

    match &tx.tx().input[..4] {
        &[169, 5, 156, 187] => {
            let (receiver, value): (Address, alloy::primitives::U256) =
                SolValue::abi_decode(&tx.tx().input[4..], true)?;
            Ok(Some(Erc20Operation::Transfer { receiver, value }))
        }
        &[35, 184, 114, 221] => {
            let (from, receiver, value): (Address, Address, alloy::primitives::U256) =
                SolValue::abi_decode(&tx.tx().input[4..], true)?;
            Ok(Some(Erc20Operation::TransferFrom {
                from,
                receiver,
                value,
            }))
        }
        &[9, 94, 167, 179] => {
            let (spender, value): (Address, alloy::primitives::U256) =
                SolValue::abi_decode(&tx.tx().input[4..], true)?;
            Ok(Some(Erc20Operation::Approve { spender, value }))
        }
        _ => Err(anyhow!("Unknown function signature")),
    }
}

pub fn handle_erc20_transfer<K: ContextKvStore>(
    chain: &mut Owshenchain<K>,
    msg_sender: Address,
    receiver: Address,
    value: Uint<256, 4>,
    token: Address,
) -> Result<()> {
    let token_decimals = chain.get_token_decimal(token)?;
    let token_symbol = chain.get_token_symbol(token)?;
    let tx_token = Token::Erc20(ERC20 {
        address: token,
        decimals: token_decimals,
        symbol: token_symbol,
    });
    let sender_balance = chain.get_balance(tx_token.clone(), msg_sender)?;

    if sender_balance >= value {
        chain.db.put(
            Key::Balance(msg_sender, tx_token.clone()),
            Some(Value::U256(sender_balance - value)),
        )?;

        let receiver_balance = chain.get_balance(tx_token.clone(), receiver)?;
        chain.db.put(
            Key::Balance(receiver, tx_token.clone()),
            Some(Value::U256(receiver_balance + value)),
        )?;

        let current_nonce = chain.get_eth_nonce(msg_sender)?;
        let incremented_nonce = current_nonce + U256::from(1);
        chain.db.put(
            Key::NonceEth(msg_sender),
            Some(Value::U256(incremented_nonce)),
        )?;

        Ok(())
    } else {
        Err(anyhow!("Insufficient balance."))
    }
}

pub fn handle_erc20_transfer_from<K: ContextKvStore>(
    chain: &mut Owshenchain<K>,
    msg_sender: Address,
    from: Address,
    receiver: Address,
    value: Uint<256, 4>,
    token: Address,
) -> Result<()> {
    let token_decimals = chain.get_token_decimal(token)?;
    let token_symbol = chain.get_token_symbol(token)?;
    let tx_token = Token::Erc20(ERC20 {
        address: token,
        decimals: token_decimals,
        symbol: token_symbol,
    });
    let allowance = chain.get_allowance(from, msg_sender, tx_token.clone())?;
    let sender_balance = chain.get_balance(tx_token.clone(), from)?;

    if sender_balance < value {
        return Err(anyhow!("Insufficient balance"));
    }

    if allowance < value {
        return Err(anyhow!("Insufficient Allowance"));
    }

    chain.db.put(
        Key::Balance(from, tx_token.clone()),
        Some(Value::U256(sender_balance - value)),
    )?;

    chain.db.put(
        Key::Allowance(from, msg_sender, tx_token.clone()),
        Some(Value::U256(allowance - value)),
    )?;

    let receiver_balance = chain.get_balance(tx_token.clone(), receiver)?;

    chain.db.put(
        Key::Balance(receiver, tx_token.clone()),
        Some(Value::U256(receiver_balance + value)),
    )?;

    let current_nonce = chain.get_eth_nonce(msg_sender)?;
    let incremented_nonce = current_nonce + U256::from(1);
    chain.db.put(
        Key::NonceEth(msg_sender),
        Some(Value::U256(incremented_nonce)),
    )?;

    Ok(())
}

pub fn handle_erc20_approve<K: ContextKvStore>(
    chain: &mut Owshenchain<K>,
    msg_sender: Address,
    spender: Address,
    value: Uint<256, 4>,
    token: Address,
) -> Result<()> {
    let token_decimals = chain.get_token_decimal(token)?;
    let token_symbol = chain.get_token_symbol(token)?;
    let tx_token = Token::Erc20(ERC20 {
        address: token,
        decimals: token_decimals,
        symbol: token_symbol,
    });
    let allowance = chain.get_allowance(msg_sender, spender, tx_token.clone())?;

    chain.db.put(
        Key::Allowance(msg_sender, spender, tx_token.clone()),
        Some(Value::U256(allowance + value)),
    )?;

    let current_nonce = chain.get_eth_nonce(msg_sender)?;
    let incremented_nonce = current_nonce + U256::from(1);
    chain.db.put(
        Key::NonceEth(msg_sender),
        Some(Value::U256(incremented_nonce)),
    )?;

    Ok(())
}
