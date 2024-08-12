use alloy::primitives::{Address, FixedBytes, U256};
use anyhow::{Ok, Result};

use crate::{
    blockchain::{Blockchain, Owshenchain},
    db::{Key, Value},
    services::ContextKvStore,
    types::Token,
};

pub fn mint_tx<K: ContextKvStore>(
    _chain: &mut Owshenchain<K>,
    _tx_hash: Vec<u8>,
    _user_tx_hash: String,
    _token: Token,
    _amount: U256,
    _address: Address,
) -> Result<(), anyhow::Error> {
    if _tx_hash.len() != 32 {
        return Err(anyhow::anyhow!("Transaction hash must be 32 bytes"));
    }

    if _chain
        .db
        .get(crate::db::Key::DepositedTransaction(_user_tx_hash))?
        .is_some()
    {
        return Err(anyhow::anyhow!("Transaction already exists"));
    }

    if _chain
        .db
        .get(Key::TransactionHash(FixedBytes::<32>::from_slice(
            &_tx_hash,
        )))?
        .is_some()
    {
        return Err(anyhow::anyhow!("Transaction already exists"));
    }

    let user_balance = _chain.get_balance(_token.clone(), _address)?;
    let new_balance = user_balance + _amount;
    _chain.db.put(
        Key::Balance(_address, _token.clone()),
        Some(Value::U256(new_balance)),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy::signers::local::PrivateKeySigner;

    use crate::{
        blockchain::Config,
        config::{CHAIN_ID, OWSHEN_CONTRACT},
        db::{KvStore, RamKvStore},
        genesis::GENESIS,
        types::{CustomTx, CustomTxMsg, IncludedTransaction, Mint},
    };

    use super::*;

    #[test]
    fn test_mint_success() {
        let conf = Config {
            chain_id: 1387,
            owner: None,
            genesis: GENESIS.clone(),
            owshen: OWSHEN_CONTRACT,
            provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
        };

        let mut chain = Owshenchain::new(conf, RamKvStore::new());

        let signer = PrivateKeySigner::random();
        let address = signer.address();

        let user_tx_hash = "0x1234567890abcdef".to_string();
        let tx_hash = vec![0u8; 32];
        let token = Token::Native;
        let amount = U256::from(100);

        let result = mint_tx(&mut chain, tx_hash, user_tx_hash, token.clone(), amount, address);
        assert!(result.is_ok());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);
    }

    #[test]
    fn test_mint_fail() {
        let conf = Config {
            chain_id: 1387,
            owner: None,
            genesis: GENESIS.clone(),
            owshen: OWSHEN_CONTRACT,
            provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
        };

        let mut chain = Owshenchain::new(conf, RamKvStore::new());

        let signer = PrivateKeySigner::random();
        let address = signer.address();

        let user_tx_hash = "0x1234567890abcdef".to_string();
        let tx_hash = vec![0u8; 32];
        let token = Token::Native;
        let amount = U256::from(100);

        let result = mint_tx(
            &mut chain,
            tx_hash.clone(),
            user_tx_hash.clone(),
            token.clone(),
            amount,
            address,
        );
        assert!(result.is_ok());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);

        let result = mint_tx(&mut chain, tx_hash, user_tx_hash, token.clone(), amount, address);
        assert!(result.is_ok());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount * U256::from(2));
    }

    #[tokio::test]
    async fn test_mint_double_spend() {
        let conf = Config {
            chain_id: 1387,
            owner: None,
            genesis: GENESIS.clone(),
            owshen: OWSHEN_CONTRACT,
            provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
        };

        let mut chain = Owshenchain::new(conf, RamKvStore::new());

        let signer = PrivateKeySigner::random();
        let address = signer.address();

        let user_tx_hash = "0x1234567890abcdef".to_string();
        let tx_hash = vec![0u8; 32];
        let token = Token::Native;
        let amount = U256::from(100);

        let result = mint_tx(
            &mut chain,
            tx_hash.clone(),
            user_tx_hash.clone(),
            token.clone(),
            amount,
            address,
        );
        assert!(result.is_ok());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);

        let tx = CustomTx::create(
            &mut signer.clone(),
            CHAIN_ID,
            CustomTxMsg::MintTx(Mint {
                tx_hash: tx_hash.clone(),
                user_tx_hash: user_tx_hash.clone(),
                token: token.clone(),
                amount,
                address,
            }),
        )
        .await
        .unwrap();
        let bincodable_tx = tx.clone().try_into().unwrap();
        let block_number = 4321;

        let included_tx = IncludedTransaction {
            tx: bincodable_tx,
            block_hash: FixedBytes::from([0u8; 32]),
            block_number,
            transaction_index: 1,
        };

        chain
            .db
            .put(
                Key::TransactionHash(FixedBytes::from_slice(&tx_hash)),
                Some(Value::Transaction(included_tx)),
            )
            .unwrap();

        let result = mint_tx(&mut chain, tx_hash, user_tx_hash, token.clone(), amount, address);
        assert!(result.is_err());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);
    }

    #[tokio::test]
    async fn test_mint_double_spend_with_same_user_transaction_hash() {
        let conf = Config {
            chain_id: 1387,
            owner: None,
            genesis: GENESIS.clone(),
            owshen: OWSHEN_CONTRACT,
            provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
        };

        let mut chain = Owshenchain::new(conf, RamKvStore::new());

        let signer = PrivateKeySigner::random();
        let address = signer.address();

        let user_tx_hash = "0x1234567890abcdef".to_string();
        let tx_hash = vec![0u8; 32];
        let token = Token::Native;
        let amount = U256::from(100);

        let result = mint_tx(
            &mut chain,
            tx_hash.clone(),
            user_tx_hash.clone(),
            token.clone(),
            amount,
            address,
        );
        assert!(result.is_ok());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);

        chain
            .db
            .put(
                Key::DepositedTransaction(user_tx_hash.clone()),
                Some(Value::DepositedTransaction(user_tx_hash.clone())),
            )
            .unwrap();

        let result = mint_tx(&mut chain, tx_hash, user_tx_hash, token.clone(), amount, address);
        assert!(result.is_err());

        let balance = chain.get_balance(token.clone(), address).unwrap();
        assert_eq!(balance, amount);
    }
}
