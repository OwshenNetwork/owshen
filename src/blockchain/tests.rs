use std::clone;

use super::*;
use crate::config;
use crate::db::{Key, KvStore, Value};
use crate::types::{network::Network, Burn, CustomTx, Token};
use crate::types::{Mint, ERC20};
use crate::{db::RamKvStore, genesis::GENESIS};
use alloy::primitives::Uint;
use alloy::primitives::{utils::parse_units, Address, U256};
use alloy::signers::local::PrivateKeySigner;
use anyhow::Ok;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[test]
fn test_block_storage() -> Result<(), anyhow::Error> {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());
    let mut tx_queue = TransactionQueue::new();

    let timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    // Then you can pass it to draft_block
    let _new_block = chain.draft_block(&mut tx_queue, timestamp).unwrap();
    assert_eq!(chain.get_height().unwrap(), 0);
    let timestamp_2: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block = chain.draft_block(&mut tx_queue, timestamp_2).unwrap();
    chain.push_block(new_block).unwrap();
    assert_eq!(chain.get_height().unwrap(), 1);
    let timestamp_3: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block = chain.draft_block(&mut tx_queue, timestamp_3).unwrap();
    chain.push_block(new_block).unwrap();
    assert_eq!(chain.get_height().unwrap(), 2);
    let timestamp_4: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block = chain.draft_block(&mut tx_queue, timestamp_4).unwrap();
    chain.push_block(new_block).unwrap();
    assert_eq!(chain.get_height().unwrap(), 3);

    assert_eq!(chain.pop_block().unwrap().unwrap().index, 2);
    assert_eq!(chain.get_height().unwrap(), 2);
    assert_eq!(chain.pop_block().unwrap().unwrap().index, 1);
    assert_eq!(chain.get_height().unwrap(), 1);
    assert_eq!(chain.pop_block().unwrap().unwrap().index, 0);
    assert_eq!(chain.get_height().unwrap(), 0);

    assert!(chain.pop_block().unwrap().is_none());
    Ok(())
}

#[test]
fn test_get_balance_without_genesis_balance() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let mock_user_address = Address::from([1; 20]);
    let mock_token: Token = Token::Erc20(ERC20 {
        address: Address::from([5;20]),
        decimals: U256::from(18),
        symbol: "USDT".to_owned(),
    });
    let mock_erc20_amount = U256::from(20);
    let mock_notive_amount = U256::from(2);

    // native
    assert_eq!(
        chain.get_balance(Token::Native, mock_user_address).unwrap(),
        U256::from(0)
    );
    chain
        .db
        .put(
            Key::Balance(mock_user_address, Token::Native),
            Some(Value::U256(mock_notive_amount)),
        )
        .unwrap();
    assert_eq!(
        chain.get_balance(Token::Native, mock_user_address).unwrap(),
        mock_notive_amount
    );

    print!(
        "amount  {:?}",
        chain
            .get_balance(mock_token.clone(), mock_user_address)
            .unwrap()
    );

    // token
    assert_eq!(
        chain
            .get_balance(mock_token.clone(), mock_user_address)
            .unwrap(),
        U256::from(0)
    );
    chain
        .db
        .put(
            Key::Balance(mock_user_address, mock_token.clone()),
            Some(Value::U256(mock_erc20_amount)),
        )
        .unwrap();
    assert_eq!(
        chain
            .get_balance(mock_token.clone(), mock_user_address)
            .unwrap(),
        mock_erc20_amount
    );
}

#[test]
fn test_get_balance_with_genesis_balance() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let genesis_mock_user_address: Address = Address::from([2; 20]);
    let genesis_token: Token = Token::Erc20(ERC20 {
        address: Address::from([3;20]),
        decimals: U256::from(18),
        symbol: "USDT".to_owned(),
    });
    let mock_genesis_native_amount = parse_units("2", 18).unwrap().into();
    let mock_genesis_token_amount = parse_units("200", 18).unwrap().into();

    // user with (native) genesis balance
    assert_eq!(
        chain
            .get_balance(Token::Native, genesis_mock_user_address)
            .unwrap(),
        mock_genesis_native_amount
    );

    // genesis native amonut +  1
    let add_native_amount: Uint<256, 4> = parse_units("1", 18).unwrap().into();
    let new_user_native_balance = mock_genesis_native_amount + add_native_amount;

    chain
        .db
        .put(
            Key::Balance(genesis_mock_user_address, Token::Native),
            Some(Value::U256(new_user_native_balance)),
        )
        .unwrap();

    assert_eq!(
        chain
            .get_balance(Token::Native, genesis_mock_user_address)
            .unwrap(),
        new_user_native_balance
    );

    // user with (token) genesis balance

    assert_eq!(
        chain
            .get_balance(genesis_token.clone(), genesis_mock_user_address)
            .unwrap(),
        mock_genesis_token_amount
    );

    // genesis token amonut +  100
    let add_amount: Uint<256, 4> = parse_units("100", 18).unwrap().into();
    let new_user_balance = mock_genesis_token_amount + add_amount;

    chain
        .db
        .put(
            Key::Balance(genesis_mock_user_address, genesis_token.clone()),
            Some(Value::U256(new_user_balance)),
        )
        .unwrap();

    assert_eq!(
        chain
            .get_balance(genesis_token.clone(), genesis_mock_user_address)
            .unwrap(),
        new_user_balance
    );
}

#[test]
fn test_get_nonce_eth() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let mock_address = Address::from([1; 20]);

    assert_eq!(chain.get_eth_nonce(mock_address).unwrap(), U256::from(0));

    chain
        .db
        .put(
            Key::NonceEth(mock_address),
            Some(Value::U256(U256::from(5))),
        )
        .unwrap();

    assert_eq!(chain.get_eth_nonce(mock_address).unwrap(), U256::from(5));
}

#[test]
fn test_get_nonce_custom() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let mock_address = Address::from([2; 20]);
    assert_eq!(chain.get_custom_nonce(mock_address).unwrap(), U256::from(0));

    chain
        .db
        .put(
            Key::NonceCustom(mock_address),
            Some(Value::U256(U256::from(10))),
        )
        .unwrap();

    assert_eq!(
        chain.get_custom_nonce(mock_address).unwrap(),
        U256::from(10)
    );
}

#[tokio::test]
async fn test_get_user_withdraw_transactions() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let signer = PrivateKeySigner::random();
    let tx = CustomTx::create(
        &mut signer.clone(),
        123,
        CustomTxMsg::BurnTx(Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: Network::ETH,
            token: Token::Native,
            amount: U256::from(100),
            calldata: None,
        }),
    )
    .await
    .unwrap();

    let bincodable_tx: BincodableOwshenTransaction = tx.clone().try_into().unwrap();

    let included_tx: IncludedTransaction = IncludedTransaction {
        tx: bincodable_tx,
        block_hash: FixedBytes::from([0u8; 32]),
        block_number: 4321,
        transaction_index: 1,
    };
    let key = Key::Transactions(signer.address());
    let mut transactions: Vec<IncludedTransaction> = Vec::new();
    transactions.push(included_tx.clone());

    chain
        .db
        .put(key, Some(Value::Transactions(transactions)))
        .unwrap();

    let withdrawals = chain.get_user_withdrawals(signer.address()).unwrap();
    assert_eq!(withdrawals.len(), 1);
    assert_eq!(withdrawals[0].tx, included_tx.tx);
    assert_eq!(withdrawals[0].block_number, included_tx.block_number);
    assert_eq!(withdrawals[0].block_hash, included_tx.block_hash);
    assert_eq!(
        withdrawals[0].transaction_index,
        included_tx.transaction_index
    );
}

#[tokio::test]
async fn test_get_transactions_per_second() -> Result<(), anyhow::Error> {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf.clone(), RamKvStore::new());
    let mut tx_queue = TransactionQueue::new();

    let tx_hash = vec![0u8; 32];

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let user_tx_hash = "0x1234567890abcdef".to_string();
    let tx1 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx1.clone());

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx2 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();
    tx_queue.enqueue(tx2.clone());
    let timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block1 = chain.draft_block(&mut tx_queue, timestamp).unwrap();
    chain.push_block(new_block1.clone()).unwrap();

    assert_eq!(chain.get_height().unwrap(), 1);

    sleep(Duration::from_secs(1)).await;

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx3 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),

            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx3);

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx4 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx4);

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx5 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx5);
    let timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block2 = chain.draft_block(&mut tx_queue, timestamp).unwrap();
    chain.push_block(new_block2.clone()).unwrap();

    assert_eq!(chain.get_height().unwrap(), 2);

    sleep(Duration::from_secs(1)).await;

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx6 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx6);

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx7 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx7);

    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let tx8 = CustomTx::create(
        &mut signer.clone(),
        conf.clone().chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash: tx_hash.clone(),
            user_tx_hash: user_tx_hash.clone(),
            token:  Token::Erc20(ERC20 {
                address: random_token_address,
                decimals: U256::from(18),
                symbol: "USDT".to_owned(),
            }),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();

    tx_queue.enqueue(tx8);
    let timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let new_block3 = chain.draft_block(&mut tx_queue, timestamp).unwrap();
    chain.push_block(new_block3.clone()).unwrap();

    assert_eq!(chain.get_height().unwrap(), 3);

    // Log the TPS calculation
    let tps = chain.get_transactions_per_second().unwrap();
    println!("TPS: {:?}", tps);
    assert!(tps > 0.0, "TPS should be greater than zero");
    Ok(())
}

#[test]
fn test_get_allowance() {
    let conf = Config {
        chain_id: 1387,
        owner: None,
        genesis: GENESIS.clone(),
        owshen: config::OWSHEN_CONTRACT,
        provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
    };
    let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());

    let mock_user_address = Address::from([1; 20]);
    let mock_spender_address = Address::from([2; 20]);
    let mock_token: Token =  Token::Erc20(ERC20 {
        address: Address::from([5; 20]),
        decimals: U256::from(18),
        symbol: "USDT".to_owned(),
    });
    let mock_erc20_amount = U256::from(20);

    assert_eq!(
        chain
            .get_allowance(mock_user_address, mock_spender_address, mock_token.clone())
            .unwrap(),
        U256::from(0)
    );

    chain
        .db
        .put(
            Key::Allowance(mock_user_address, mock_spender_address, mock_token.clone()),
            Some(Value::U256(mock_erc20_amount)),
        )
        .unwrap();
    assert_eq!(
        chain
            .get_allowance(mock_user_address, mock_spender_address, mock_token.clone())
            .unwrap(),
        mock_erc20_amount
    );
}
