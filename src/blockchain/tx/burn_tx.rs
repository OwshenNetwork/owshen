use alloy::primitives::{Address, U256};
use anyhow::Result;

use crate::{
    blockchain::{Blockchain, Owshenchain},
    db::{Key, KvStore, Value},
    services::ContextKvStore,
    types::{Burn, Token, WithdrawCalldata},
};

pub fn burn_tx<K: ContextKvStore>(_chain: &mut Owshenchain<K>, _data: Burn) -> Result<()> {
    let address = match _data.calldata {
        Some(WithdrawCalldata::Eth { address }) => address,
        _ => return Err(anyhow::anyhow!("Invalid calldata!")),
    };

    if _chain.db.get(Key::BurnId(_data.burn_id.clone()))?.is_some() {
        return Err(anyhow::anyhow!("Burn id already used!"));
    }

    let user_balance = _chain.get_balance(_data.token.clone(), address)?;
    if user_balance < _data.amount {
        return Err(anyhow::anyhow!("Insufficient balance!"));
    }

    _chain.db.put(
        Key::Balance(address, _data.token.clone()),
        Some(Value::U256(user_balance - _data.amount)),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy::{primitives::FixedBytes, signers::local::PrivateKeySigner};

    use crate::{
        blockchain::Config, config::OWSHEN_CONTRACT, db::RamKvStore, genesis::GENESIS,
        types::network::Network,
    };

    use super::*;

    #[test]
    fn test_burn_tx_success() {
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

        chain
            .db
            .put(
                Key::Balance(address, Token::Native),
                Some(Value::U256(U256::from(1000))),
            )
            .unwrap();

        let data = Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: Network::ETH,
            token: Token::Native,
            amount: U256::from(100),
            calldata: Some(WithdrawCalldata::Eth { address }),
        };

        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(1000)
        );
        burn_tx(&mut chain, data).unwrap();
        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(900)
        );
    }

    #[test]
    fn test_burn_tx_fail_already_used_burn_id() {
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

        chain
            .db
            .put(
                Key::Balance(address, Token::Native),
                Some(Value::U256(U256::from(1000))),
            )
            .unwrap();

        let data = Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: Network::ETH,
            token: Token::Native,
            amount: U256::from(100),
            calldata: Some(WithdrawCalldata::Eth { address }),
        };

        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(1000)
        );
        burn_tx(&mut chain, data.clone()).unwrap();
        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(900)
        );

        chain
            .db
            .put(Key::BurnId(data.burn_id.clone()), Some(Value::Void))
            .unwrap();
        assert!(burn_tx(&mut chain, data).is_err());
        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(900)
        );
    }

    #[test]
    fn test_burn_tx_insufficient_balance() {
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

        chain
            .db
            .put(
                Key::Balance(address, Token::Native),
                Some(Value::U256(U256::from(1000))),
            )
            .unwrap();

        let data = Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: Network::ETH,
            token: Token::Native,
            amount: U256::from(1001),
            calldata: Some(WithdrawCalldata::Eth { address }),
        };

        assert_eq!(
            chain.get_balance(Token::Native, address).unwrap(),
            U256::from(1000)
        );
        assert!(burn_tx(&mut chain, data).is_err());
    }
}
