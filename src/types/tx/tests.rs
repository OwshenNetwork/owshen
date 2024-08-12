use super::*;
use crate::types::{Token, ERC20};

use alloy::{primitives::U256, signers::local::PrivateKeySigner};

#[tokio::test]
async fn test_mint_tx_native() {
    let chain_id = 2341;
    let tx_hash = vec![0u8; 32];
    let signer = PrivateKeySigner::random();
    let user_tx_hash = "0x1234567890abcdef".to_string();
    let tx = CustomTx::create(
        &mut signer.clone(),
        chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash,
            user_tx_hash,
            token: Token::Native,
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(tx.signer().unwrap(), signer.address());
    match tx {
        OwshenTransaction::Custom(custom_tx) => match custom_tx.msg().unwrap() {
            CustomTxMsg::MintTx(mint) => {
                assert_eq!(mint.tx_hash, vec![0u8; 32]);
                assert_eq!(mint.user_tx_hash, "0x1234567890abcdef");
                assert_eq!(mint.token, Token::Native);
                assert_eq!(mint.amount, U256::from(100));
                assert_eq!(mint.address, signer.address());
            }
            _ => panic!("Invalid tx!"),
        },
        _ => panic!("Invalid tx!"),
    }
}

#[tokio::test]
async fn test_mint_tx_erc20() {
    let chain_id = 2341;
    let tx_hash = vec![0u8; 32];
    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let user_tx_hash = "0x1234567890abcdef".to_string();

    let token = Token::Erc20(ERC20 {
        address: random_token_address,
        decimals: U256::from(18),
        symbol: "USDT".to_owned(),
    });

    let tx = CustomTx::create(
        &mut signer.clone(),
        chain_id,
        CustomTxMsg::MintTx(Mint {
            tx_hash,
            user_tx_hash,
            token: token.clone(),
            amount: U256::from(100),
            address: signer.address(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(tx.signer().unwrap(), signer.address());
    match tx {
        OwshenTransaction::Custom(custom_tx) => match custom_tx.msg().unwrap() {
            CustomTxMsg::MintTx(mint) => {
                assert_eq!(mint.tx_hash, vec![0u8; 32]);
                assert_eq!(mint.user_tx_hash, "0x1234567890abcdef");
                assert_eq!(mint.token, token.clone());
                assert_eq!(mint.amount, U256::from(100));
                assert_eq!(mint.address, signer.address());
            }
            _ => panic!("Invalid tx!"),
        },
        _ => panic!("Invalid tx!"),
    }
}

#[tokio::test]
async fn test_burn_tx_native() {
    let chain_id = 2341;
    let signer = PrivateKeySigner::random();
    let tx = CustomTx::create(
        &mut signer.clone(),
        chain_id,
        CustomTxMsg::BurnTx(Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: crate::types::network::Network::BSC,
            token: Token::Native,
            amount: U256::from(100),
            calldata: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(tx.signer().unwrap(), signer.address());
    match tx {
        OwshenTransaction::Custom(custom_tx) => match custom_tx.msg().unwrap() {
            CustomTxMsg::BurnTx(burn) => {
                assert_eq!(burn.token, Token::Native);
                assert_eq!(burn.amount, U256::from(100));
            }
            _ => panic!("Invalid tx!"),
        },
        _ => panic!("Invalid tx!"),
    }
}

#[tokio::test]
async fn test_burn_tx_erc20() {
    let chain_id = 2341;
    let signer = PrivateKeySigner::random();
    let random_token_address = PrivateKeySigner::random().address();
    let token = Token::Erc20(ERC20 {
        address: random_token_address,
        decimals: U256::from(18),
        symbol: "USDT".to_owned(),
    });
    let tx = CustomTx::create(
        &mut signer.clone(),
        chain_id,
        CustomTxMsg::BurnTx(Burn {
            burn_id: FixedBytes::from([1u8; 32]),
            network: crate::types::network::Network::BSC,
            token: token.clone(),
            amount: U256::from(100),
            calldata: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(tx.signer().unwrap(), signer.address());
    match tx {
        OwshenTransaction::Custom(custom_tx) => match custom_tx.msg().unwrap() {
            CustomTxMsg::BurnTx(burn) => {
                assert_eq!(burn.token, token.clone());
                assert_eq!(burn.amount, U256::from(100));
            }
            _ => panic!("Invalid tx!"),
        },
        _ => panic!("Invalid tx!"),
    }
}
