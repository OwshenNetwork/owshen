use std::{ops::Add, rc::Rc, sync::Arc};
use tokio::sync::Mutex;

use alloy::{
    consensus::{Transaction, TxEip1559, TxEnvelope, TypedTransaction},
    network::{Ethereum, EthereumWallet, NetworkWallet},
    primitives::{keccak256, Address, Bytes, TxKind, Uint, B256, U256},
    rlp::Decodable,
    rpc::types::{AccessList, AccessListItem},
    signers::local::PrivateKeySigner,
    sol_types::SolValue,
};
use alloy_sol_types::abi::encode;

use anyhow::{anyhow, Result};

use evm::{Capture, ExitReason, Runtime};

use crate::{
    blockchain::{tx::erc20::*, Blockchain, Config, Owshenchain, TransactionQueue},
    config,
    db::{Key, KvStore, RamKvStore, Value},
    genesis::GENESIS,
    services::{Context, ContextKvStore},
    types::{Token, ERC20},
};

pub fn eth<K: ContextKvStore>(
    _chain: &mut Owshenchain<K>,
    _msg_sender: Address,
    _tx: &TxEnvelope,
) -> Result<()> {
    log::info!("Ethereum transaction, by {}!", _msg_sender);
    let tx = _tx
        .as_eip1559()
        .ok_or(anyhow!("Only EIP1559 transactions are supported!"))?;
    match tx.tx().to {
        TxKind::Create => {
            return Err(anyhow!("Contract creation is not supported."));
            // let bytecode = tx.tx().input.to_vec();
            // let mut ovm = Ovm::new(chain);
            // let ctx = Context {
            //     address: H160::random(),
            //     apparent_value: U256::from(0),
            //     caller: H160::random(),
            // };
            // let mut runtime = Runtime::new(Rc::new(bytecode), Rc::new(vec![]), ctx, 10000, 10000);
            // match runtime.run(&mut ovm) {
            //     Capture::Exit(e) => match e {
            //         ExitReason::Succeed(_s) => {
            //             let contract_code = runtime.machine().return_value();
            //             let contract_address = Address::from_slice(
            //                 &keccak256((by, chain.get_eth_nonce(by)?).abi_encode()).0[12..],
            //             );
            //             chain.db.put(
            //                 Key::ContractCode(contract_address),
            //                 Some(Value::VecU8(contract_code)),
            //             )?;
            //             println!("Contract deployed on {}!", contract_address);
            //         }
            //         _ => {
            //             return Err(anyhow!("Failed!"));
            //         }
            //     },
            //     Capture::Trap(_) => {
            //         return Err(anyhow!("Trapped!"));
            //     }
            // }
        }
        TxKind::Call(to) => {
            let bytecode = _chain.db.get(Key::ContractCode(to))?;
            if let Some(bytecode) = bytecode {
                return Err(anyhow!("Contract calls are not supported."));

                // let bytecode = bytecode.as_vec_u8()?;
                // let mut ovm = Ovm::new(chain);
                // let ctx = Context {
                //     address: H160::random(),
                //     apparent_value: U256::from(0),
                //     caller: H160::random(),
                // };
                // let mut runtime = Runtime::new(
                //     Rc::new(bytecode),
                //     Rc::new(tx.tx().input.to_vec()),
                //     ctx,
                //     10000,
                //     10000,
                // );
                // match runtime.run(&mut ovm) {
                //     Capture::Exit(e) => match e {
                //         ExitReason::Succeed(_s) => {
                //             let _ret_value = runtime.machine().return_value();
                //         }
                //         _ => {
                //             return Err(anyhow!("Failed!"));
                //         }
                //     },
                //     Capture::Trap(_) => {
                //         return Err(anyhow!("Trapped!"));
                //     }
                // }
            } else {
                let transaction = extract_erc20_transfer(&_tx)?;
                match transaction {
                    Some(Erc20Operation::Transfer { receiver, value }) => {
                        handle_erc20_transfer(_chain, _msg_sender, receiver, value, to)?;
                    }
                    Some(Erc20Operation::TransferFrom {
                        from,
                        receiver,
                        value,
                    }) => {
                        handle_erc20_transfer_from(_chain, _msg_sender, from, receiver, value, to)?
                    }
                    Some(Erc20Operation::Approve { spender, value }) => {
                        handle_erc20_approve(_chain, _msg_sender, spender, value, to)?
                    }
                    None => {
                        let value = tx.tx().value();
                        let sender_balance = _chain.get_balance(Token::Native, _msg_sender)?;

                        if sender_balance >= value {
                            _chain.db.put(
                                Key::Balance(_msg_sender, Token::Native),
                                Some(Value::U256(sender_balance - value)),
                            )?;

                            let privious_receiver_balance =
                                _chain.get_balance(Token::Native, to)?;

                            _chain.db.put(
                                Key::Balance(to, Token::Native),
                                Some(Value::U256(privious_receiver_balance + value)),
                            )?;
                            let current_nonce = _chain.get_eth_nonce(_msg_sender)?;
                            let incremented_nonce = current_nonce + U256::from(1);
                            _chain.db.put(
                                Key::NonceEth(_msg_sender),
                                Some(Value::U256(incremented_nonce)),
                            )?;
                        } else {
                            return Err(anyhow!("Insufficient balance."));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn setup_mock_chain() -> Owshenchain<RamKvStore> {
    let mock_db = RamKvStore::new();
    return Owshenchain {
        db: mock_db.clone(),
        config: Config {
            chain_id: 1387,
            owner: None,
            genesis: GENESIS.clone(),
            owshen: config::OWSHEN_CONTRACT,
            provider_address: "http://127.0.0.1:8888".parse().expect("faild to parse"),
        },
    };
}

#[tokio::test]
async fn test_erc20_transfer() {
    let mut chain = setup_mock_chain();
    let msg_sender = Address::from([1; 20]);
    let receiver = Address::from([2; 20]);
    let token_contract = Address::from([6; 20]);

    let transaction_value = Uint::<256, 4>::from(0);

    let token_decimals = chain.get_token_decimal(token_contract).unwrap();
    let token_symbol = chain.get_token_symbol(token_contract).unwrap();
    let tx_token = Token::Erc20(ERC20 {
        address: token_contract,
        decimals: token_decimals,
        symbol: token_symbol,
    });

    let _ = chain.db.put(
        Key::Balance(msg_sender, tx_token.clone()),
        Some(Value::U256(U256::from(100000))),
    );

    let sender_pre_transaction_balance = chain.get_balance(tx_token.clone(), msg_sender).unwrap();
    let receiver_pre_transaction_balance = chain.get_balance(tx_token.clone(), receiver).unwrap();

    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let transfer_method = [169, 5, 156, 187];

    let mut data = Vec::new();
    let re_encoded = receiver.abi_encode();
    let val_encoded = transaction_value.abi_encode();
    data.extend_from_slice(&transfer_method);
    data.extend_from_slice(&re_encoded);
    data.extend_from_slice(&val_encoded);

    let tx = TxEip1559 {
        nonce: 0,
        gas_limit: 21_000,
        to: TxKind::Call(token_contract),
        value: Uint::<256, 4>::from(0),
        input: Bytes::from(data.clone()),
        chain_id: chain.config.chain_id,
        max_priority_fee_per_gas: 3_000_000,
        max_fee_per_gas: 300_000_000,
        access_list: AccessList(vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![B256::ZERO],
        }]),
    };

    let typed_tx = TypedTransaction::Eip1559(tx.clone());

    let signed_tx =
        <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction(&wallet, typed_tx)
            .await
            .unwrap();

    let pre_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    let result = eth(&mut chain, msg_sender, &signed_tx);
    assert!(result.is_ok());

    let sender_post_transaction_balance = chain.get_balance(tx_token.clone(), msg_sender).unwrap();
    let receiver_post_transaction_balance = chain.get_balance(tx_token.clone(), receiver).unwrap();
    let post_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    assert_eq!(
        sender_post_transaction_balance,
        sender_pre_transaction_balance - transaction_value
    );
    assert_eq!(
        receiver_post_transaction_balance,
        receiver_pre_transaction_balance + transaction_value
    );

    assert_eq!(post_tx_nonce, pre_tx_nonce + U256::from(1));
    assert_eq!(tx.value().clone(), U256::from(0));
}

#[tokio::test]
async fn test_erc20_approve() {
    let mut chain = setup_mock_chain();
    let wallet: EthereumWallet = EthereumWallet::new(PrivateKeySigner::random());

    let owner: Address =
        <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

    let spender = Address::from([2; 20]);
    let token_contract = Address::from([6; 20]);

    let token_decimals = chain.get_token_decimal(token_contract).unwrap();
    let token_symbol = chain.get_token_symbol(token_contract).unwrap();
    let tx_token = Token::Erc20(ERC20 {
        address: token_contract,
        decimals: token_decimals,
        symbol: token_symbol,
    });

    let transaction_value = Uint::<256, 4>::from(0);

    let spender_pre_transaction_allowance = chain
        .get_allowance(owner, spender, tx_token.clone())
        .unwrap();

    let approve_method = [9, 94, 167, 179];

    let mut data = Vec::new();
    let spender_encoded = spender.abi_encode();
    let val_encoded = transaction_value.abi_encode();
    data.extend_from_slice(&approve_method);
    data.extend_from_slice(&spender_encoded);
    data.extend_from_slice(&val_encoded);

    let tx = TxEip1559 {
        nonce: 0,
        gas_limit: 21_000,
        to: TxKind::Call(token_contract),
        value: Uint::<256, 4>::from(0),
        input: Bytes::from(data.clone()),
        chain_id: chain.config.chain_id,
        max_priority_fee_per_gas: 3_000_000,
        max_fee_per_gas: 300_000_000,
        access_list: AccessList(vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![B256::ZERO],
        }]),
    };

    let typed_tx = TypedTransaction::Eip1559(tx);

    let signed_tx =
        <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction(&wallet, typed_tx)
            .await
            .unwrap();

    let pre_tx_nonce = chain.get_eth_nonce(owner).unwrap();

    let result = eth(&mut chain, owner, &signed_tx);

    assert!(result.is_ok());

    let post_tx_nonce = chain.get_eth_nonce(owner).unwrap();

    let spender_post_transaction_allowance = chain
        .get_allowance(owner, spender, tx_token.clone())
        .unwrap();

    assert_eq!(
        spender_post_transaction_allowance,
        spender_pre_transaction_allowance + transaction_value
    );

    assert_eq!(post_tx_nonce, pre_tx_nonce + U256::from(1));
}

#[tokio::test]
async fn test_erc20_transfer_from() {
    let mut chain = setup_mock_chain();
    let wallet: EthereumWallet = EthereumWallet::new(PrivateKeySigner::random());
    let msg_sender: Address =
        <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

    let receiver = Address::from([2; 20]);
    let from = Address::from([3; 20]);
    let token_contract = Address::from([6; 20]);

    let token_decimals = chain.get_token_decimal(token_contract).unwrap();
    let token_symbol = chain.get_token_symbol(token_contract).unwrap();
    let tx_token = Token::Erc20(ERC20 {
        address: token_contract,
        decimals: token_decimals,
        symbol: token_symbol,
    });

    let transaction_value = Uint::<256, 4>::from(0);

    let _ = chain.db.put(
        Key::Balance(from, tx_token.clone()),
        Some(Value::U256(U256::from(100000))),
    );

    let _ = chain.db.put(
        Key::Allowance(from, msg_sender, tx_token.clone()),
        Some(Value::U256(U256::from(100000))),
    );

    let from_pre_transaction_balance = chain.get_balance(tx_token.clone(), from).unwrap();
    let receiver_pre_transaction_balance = chain.get_balance(tx_token.clone(), receiver).unwrap();
    let msg_sender_pre_transaction_allowance = chain
        .get_allowance(from, msg_sender, tx_token.clone())
        .unwrap();
    let pre_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    let transfer_from_method = [35, 184, 114, 221];

    let mut data = Vec::new();
    let from_encoded = from.abi_encode();
    let receiver_encoded = receiver.abi_encode();
    let val_encoded = transaction_value.abi_encode();
    data.extend_from_slice(&transfer_from_method);
    data.extend_from_slice(&from_encoded);
    data.extend_from_slice(&receiver_encoded);
    data.extend_from_slice(&val_encoded);

    let tx = TxEip1559 {
        nonce: 0,
        gas_limit: 21_000,
        to: TxKind::Call(token_contract),
        value: Uint::<256, 4>::from(0),
        input: Bytes::from(data.clone()),
        chain_id: chain.config.chain_id,
        max_priority_fee_per_gas: 3_000_000,
        max_fee_per_gas: 300_000_000,
        access_list: AccessList(vec![
            AccessListItem {
                address: Address::ZERO,
                storage_keys: vec![B256::ZERO],
            },
            AccessListItem {
                address: Address::ZERO,
                storage_keys: vec![B256::ZERO],
            },
        ]),
    };

    let typed_tx = TypedTransaction::Eip1559(tx.clone());

    let signed_tx =
        <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction(&wallet, typed_tx)
            .await
            .unwrap();

    let result = eth(&mut chain, msg_sender, &signed_tx);
    assert!(result.is_ok());
    let post_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    let from_post_transaction_balance = chain.get_balance(tx_token.clone(), from).unwrap();
    let receiver_post_transaction_balance = chain.get_balance(tx_token.clone(), receiver).unwrap();

    let msg_sender_post_transaction_allowance = chain
        .get_allowance(from, msg_sender, tx_token.clone())
        .unwrap();

    assert_eq!(
        from_post_transaction_balance,
        from_pre_transaction_balance - transaction_value
    );
    assert_eq!(
        receiver_post_transaction_balance,
        receiver_pre_transaction_balance + transaction_value
    );

    assert_eq!(
        msg_sender_post_transaction_allowance,
        msg_sender_pre_transaction_allowance - transaction_value
    );
    assert_eq!(post_tx_nonce, pre_tx_nonce + U256::from(1));
    assert_eq!(tx.value().clone(), U256::from(0));
}

#[tokio::test]
async fn test_eth_transfer() {
    let mut chain = setup_mock_chain();
    let wallet: EthereumWallet = EthereumWallet::new(PrivateKeySigner::random());
    let msg_sender: Address =
        <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

    let receiver = Address::from([3; 20]);
    let transaction_value = Uint::<256, 4>::from(1000);

    let _ = chain.db.put(
        Key::Balance(msg_sender, Token::Native),
        Some(Value::U256(U256::from(100000))),
    );

    let from_pre_transaction_balance = chain.get_balance(Token::Native, msg_sender).unwrap();
    let receiver_pre_transaction_balance = chain.get_balance(Token::Native, receiver).unwrap();
    let pre_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    let tx = TxEip1559 {
        nonce: 0,
        gas_limit: 21_000,
        to: TxKind::Call(receiver),
        value: Uint::<256, 4>::from(1000),
        input: Bytes::new(),
        chain_id: chain.config.chain_id,
        max_priority_fee_per_gas: 3_000_000,
        max_fee_per_gas: 300_000_000,
        access_list: AccessList(vec![AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![B256::ZERO],
        }]),
    };

    let typed_tx = TypedTransaction::Eip1559(tx);

    let signed_tx =
        <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction(&wallet, typed_tx)
            .await
            .unwrap();

    let result = eth(&mut chain, msg_sender, &signed_tx);
    assert!(result.is_ok());

    let from_post_transaction_balance = chain.get_balance(Token::Native, msg_sender).unwrap();
    let receiver_post_transaction_balance = chain.get_balance(Token::Native, receiver).unwrap();
    let post_tx_nonce = chain.get_eth_nonce(msg_sender).unwrap();

    assert_eq!(
        from_post_transaction_balance,
        from_pre_transaction_balance - transaction_value
    );
    assert_eq!(
        receiver_post_transaction_balance,
        receiver_pre_transaction_balance + transaction_value
    );
    assert_eq!(post_tx_nonce, pre_tx_nonce + U256::from(1));
}
