use std::collections::HashMap;
use std::sync::Arc;

use super::Context;
use crate::blockchain::Blockchain;
use crate::db::{Key, KvStore, Value};
use crate::services::rpc_services::test_config;
use crate::services::{ContextKvStore, ContextSigner};
use crate::types::{Token, ERC20};
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, U256};
use alloy::sol_types::SolValue;
use anyhow::{anyhow, Result};
use hex;
use jsonrpsee::types::Params;
use serde_json::json;
use tokio::sync::Mutex;

pub async fn eth_call<S: ContextSigner, K: ContextKvStore>(
    ctx: Arc<Arc<Mutex<Context<S, K>>>>,
    params: Params<'static>,
) -> Result<String> {
    let first_param: HashMap<String, String> = params.sequence().next()?;
    let data = first_param
        .get("data")
        .ok_or(anyhow!("Data unavailable!"))?;
    let contract_address: Address = first_param
        .get("to")
        .ok_or(anyhow!("Contract address unavailable!"))?
        .parse()?;
    let method_hash = &data[0..10];
    match method_hash {
        "0x01ffc9a7" => {
            return Ok(format!("0x{:x}", 1));
        }
        // balanceOf(address)
        "0x70a08231" => {
            let address: Address = data[10..].parse()?;
            let decimals = ctx.lock().await.chain.get_token_decimal(contract_address)?;
            let symbol = ctx.lock().await.chain.get_token_symbol(contract_address)?;
            let token = Token::Erc20(ERC20 {
                address: contract_address,
                decimals,
                symbol,
            });

            let balance = ctx.lock().await.chain.get_balance(token, address)?;
            return Ok((balance).abi_encode().encode_hex());
        }
        // decimals()
        "0x313ce567" => {
            let decimals = ctx.lock().await.chain.get_token_decimal(contract_address)?;
            return Ok((decimals).abi_encode().encode_hex());
        }
        // symbol()
        "0x95d89b41" => {
            let symbol = ctx.lock().await.chain.get_token_symbol(contract_address)?;
            return Ok((symbol).abi_encode().encode_hex());
        }
        _ => {
            log::warn!("Unknown method_hash: {}", method_hash);
        }
    }
    Ok(format!("0x{:x}", 0))
}

#[tokio::test]
async fn test_eth_call() {
    let _ctx = test_config().await;

    let contract_address: Address = Address::from([7; 20]);
    let balance = U256::from(100);
    let token_symbol = "USDT".to_owned();
    let token_decimals = U256::from(18);

    _ctx.lock()
        .await
        .chain
        .db
        .put(
            Key::TokenDecimal(contract_address),
            Some(Value::U256(token_decimals)),
        )
        .unwrap();

    _ctx.lock()
        .await
        .chain
        .db
        .put(
            Key::TokenSymbol(contract_address),
            Some(Value::Symbol(token_symbol.clone())),
        )
        .unwrap();

    {
        let method_hash = "0x70a08231";
        let address: Address = Address::from([8; 20]);
        let data = format!("{}{}", method_hash, hex::encode(address));
        let addr = contract_address.to_string();
        let param_map = json!([{
            "to": addr,
            "data": data
        }])
        .to_string();
        let addr_static: &'static str = Box::leak(param_map.into_boxed_str());
        let params = Params::new(Some(addr_static));

        _ctx.lock()
            .await
            .chain
            .db
            .put(
                Key::Balance(
                    address,
                    Token::Erc20(ERC20 {
                        address: contract_address,
                        decimals: token_decimals,
                        symbol: token_symbol.clone(),
                    }),
                ),
                Some(Value::U256(balance)),
            )
            .unwrap();

        let result = eth_call(_ctx.clone().into(), params).await;
        assert!(result.is_ok());
        let expected_result = balance.abi_encode().encode_hex();
        assert_eq!(result.unwrap(), expected_result);
    }

    {
        let method_hash = "0x313ce567";
        let address: Address = Address::from([8; 20]);

        let data = format!("{}{}", method_hash, hex::encode(address));
        let addr = contract_address.to_string();
        let param_map = json!([{
            "to": addr,
            "data": data
        }])
        .to_string();
        let addr_static: &'static str = Box::leak(param_map.into_boxed_str());
        let params = Params::new(Some(addr_static));

        let result = eth_call(_ctx.clone().into(), params).await;

        assert!(result.is_ok());
        let expected_result = token_decimals.abi_encode().encode_hex();
        assert_eq!(result.unwrap(), expected_result);
    }

    {
        let method_hash = "0x95d89b41";
        let address: Address = Address::from([8; 20]);

        let data = format!("{}{}", method_hash, hex::encode(address));
        let addr = contract_address.to_string();
        let param_map = json!([{
            "to": addr,
            "data": data
        }])
        .to_string();
        let addr_static: &'static str = Box::leak(param_map.into_boxed_str());
        let params = Params::new(Some(addr_static));

        let result = eth_call(_ctx.clone().into(), params).await;

        assert!(result.is_ok());
        let expected_result = token_symbol.abi_encode().encode_hex();
        assert_eq!(result.unwrap(), expected_result);
    }

    {
        let method_hash = "0x12345678";
        let address: Address = Address::from([8; 20]);

        let data = format!("{}{}", method_hash, hex::encode(address));
        let addr = contract_address.to_string();
        let param_map = json!([{
            "to": addr,
            "data": data
        }])
        .to_string();
        let addr_static: &'static str = Box::leak(param_map.into_boxed_str());
        let params = Params::new(Some(addr_static));

        let result = eth_call(_ctx.clone().into(), params).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), format!("0x{:x}", 0));
    }
}
