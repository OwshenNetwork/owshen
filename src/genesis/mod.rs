use crate::types::{Token, ERC20};
use alloy::primitives::{utils::parse_units, Address, U256};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct Balance {
    address: String,
    amount: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenData {
    token_type: String,
    contract_address: Option<String>,
    decimal: Option<U256>,
    symbol: Option<String>,
    balances: Vec<Balance>,
}

#[derive(Debug, Clone)]
pub struct Genesis {
    pub tokens: HashMap<Token, HashMap<Address, U256>>,
}

impl Genesis {
    pub fn new(tokens: Vec<(Token, Vec<(Address, U256)>)>) -> Self {
        let mut tokens_map = HashMap::new();

        for (token, balances) in tokens {
            let mut balance_map = HashMap::new();
            for (addr, bal) in balances {
                balance_map.insert(addr, bal);
            }
            tokens_map.insert(token, balance_map);
        }

        Genesis { tokens: tokens_map }
    }
}

fn init_genesis_from_json(json_file_path: &str) -> Result<Genesis, Box<dyn std::error::Error>> {
    let file = File::open(json_file_path)?;
    let reader = BufReader::new(file);

    let token_data_list: Vec<TokenData> = serde_json::from_reader(reader)?;

    let mut tokens = Vec::new();

    for token_data in token_data_list {
        let token = match token_data.token_type.as_str() {
            "Native" => Token::Native,
            "Erc20" => {
                if let (Some(contract_address), Some(decimal), Some(symbol)) = (
                    token_data.contract_address.clone(),
                    token_data.decimal,
                    token_data.symbol.clone(),
                ) {
                    Token::Erc20(ERC20 {
                        address: contract_address.parse()?,
                        decimals: decimal,
                        symbol: symbol,
                    })
                } else {
                    return Err("ERC20 token missing contract address".into());
                }
            }
            _ => return Err("Unknown token type".into()),
        };

        let balances = token_data
            .balances
            .into_iter()
            .map(|balance| {
                let address = balance.address.parse()?;
                let amount = parse_units(&balance.amount, 18)?.into();
                Ok((address, amount))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

        tokens.push((token, balances));
    }

    Ok(Genesis::new(tokens))
}

lazy_static::lazy_static! {
    pub static ref GENESIS: Arc<Genesis> = {
        Arc::new(init_genesis_from_json("GENESIS.json").expect("Failed to initialize Genesis from JSON"))
    };
}
