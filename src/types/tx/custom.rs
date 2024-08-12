use alloy::primitives::FixedBytes;
use alloy::sol_types::SolValue;
use alloy::{
    primitives::{Address, U256},
    signers::{Signature, Signer},
};
use anyhow::{anyhow, Result};
use rlp::{DecoderError, RlpStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

use crate::types::ERC20;
use crate::types::{network::Network, Token};

use super::OwshenTransaction;

// TODO: OwshenAirdrop transaction (Should contain "Owshen address" and "Owshen signature")
// TODO: Mint transaction (Should contain chain_id (Depsotior Network-id) and tx_hash (Deposit TxHash))
// TODO: Burn transaction (Should contain chain_id (Withdrawer Network-id) and withdraw_sig (Withdrawal signature))

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mint {
    pub tx_hash: Vec<u8>,
    pub user_tx_hash: String,
    pub token: Token,
    pub amount: U256,
    pub address: Address,
}

impl rlp::Encodable for Mint {
    fn rlp_append(&self, s: &mut RlpStream) {
        match &self.token {
            Token::Native => {
                s.begin_list(6);
                s.append(&"mint");
                s.append(&"native");
                s.append(&self.tx_hash);
                s.append(&self.user_tx_hash);
                s.append(&self.amount.as_le_bytes().to_vec());
                s.append(&self.address.to_vec());
            }
            Token::Erc20(ERC20 {
                address,
                decimals,
                symbol,
            }) => {
                s.begin_list(9);
                s.append(&"mint");
                s.append(&"erc20");
                s.append(&self.tx_hash);
                s.append(&self.user_tx_hash);
                s.append(&self.amount.as_le_bytes().to_vec());
                s.append(&self.address.to_vec());
                s.append(&address.to_vec());
                s.append(&decimals.as_le_bytes().to_vec());
                s.append(&symbol.as_str());
            }
        }
    }
}

impl rlp::Decodable for Mint {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let token_type: String = rlp.val_at(1)?;
        let tx_hash: Vec<u8> = rlp.val_at(2)?;
        let user_tx_hash: String = rlp.val_at(3)?;
        let amount: Vec<u8> = rlp.val_at(4)?;
        let address: Vec<u8> = rlp.val_at(5)?;
        let token = match token_type.as_str() {
            "native" => Token::Native,
            "erc20" => {
                let token_address: Vec<u8> = rlp.val_at(6)?;
                let token_decimals: Vec<u8> = rlp.val_at(7)?;
                let token_symbol: String = rlp.val_at(8)?;
                let token_address = Address::from_slice(&token_address);
                Token::Erc20(ERC20 {
                    address: token_address,
                    decimals: U256::from_le_slice(&token_decimals),
                    symbol: token_symbol,
                })
            }
            _ => return Err(rlp::DecoderError::RlpExpectedToBeData),
        };
        Ok(Mint {
            tx_hash,
            user_tx_hash,
            token,
            amount: U256::from_le_slice(&amount),
            address: Address::from_slice(&address),
        })
    }
}

// msg.sender, _tokenAddress, _amount, _id, block.chainid

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WithdrawCalldata {
    Eth { address: Address },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Burn {
    pub burn_id: FixedBytes<32>,
    pub network: Network,
    pub token: Token,
    pub amount: U256,
    pub calldata: Option<WithdrawCalldata>,
}

impl rlp::Encodable for Burn {
    fn rlp_append(&self, s: &mut RlpStream) {
        let dl = if self.calldata.is_some() { 2 } else { 0 };
        match &self.token {
            Token::Native => {
                s.begin_list(4 + dl);
                s.append(&"burn");
                s.append(&self.burn_id.to_vec());
                s.append(&"native");
                s.append(&self.amount.as_le_bytes().to_vec());
            }
            Token::Erc20(ERC20 {
                address,
                decimals,
                symbol,
            }) => {
                s.begin_list(7 + dl);
                s.append(&"burn");
                s.append(&self.burn_id.to_vec());
                s.append(&"erc20");
                s.append(&self.amount.as_le_bytes().to_vec());
                s.append(&address.to_vec());
                s.append(&decimals.as_le_bytes().to_vec());
                s.append(&symbol.as_str());
            }
        }
        let network = match self.network {
            Network::ETH => "eth",
            Network::BSC => "bsc",
        };
        s.append(&network);

        if let Some(calldata) = &self.calldata {
            match calldata {
                WithdrawCalldata::Eth { address } => {
                    s.append(&address.to_vec());
                }
            }
        }
    }
}

impl rlp::Decodable for Burn {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let burn_id: Vec<u8> = rlp.val_at(1)?;
        let token_type: String = rlp.val_at(2)?;
        let amount: Vec<u8> = rlp.val_at(3)?;
        let network_idx;
        let calldata_idx;
        let token = match token_type.as_str() {
            "native" => {
                network_idx = 4;
                calldata_idx = 5;
                Token::Native
            }
            "erc20" => {
                network_idx = 7;
                calldata_idx = 8;
                let address: Vec<u8> = rlp.val_at(4)?;
                let token_decimals: Vec<u8> = rlp.val_at(5)?;
                let token_symbol: String = rlp.val_at(6)?;
                let token_address = Address::from_slice(&address);
                Token::Erc20(ERC20 {
                    address: token_address,
                    decimals: U256::from_le_slice(&token_decimals),
                    symbol: token_symbol,
                })
            }
            _ => return Err(rlp::DecoderError::RlpExpectedToBeData),
        };
        let network: String = rlp.val_at(network_idx)?;
        let network = match network.as_str() {
            "eth" => Network::ETH,
            "bsc" => Network::BSC,
            _ => return Err(rlp::DecoderError::RlpExpectedToBeData),
        };

        let address: Result<Vec<u8>, _> = rlp.val_at(calldata_idx);
        match address {
            Ok(address) => {
                let address = Address::from_slice(&address);
                let calldata = Some(WithdrawCalldata::Eth { address });
                return Ok(Burn {
                    burn_id: FixedBytes::from_slice(&burn_id),
                    network,
                    token,
                    amount: U256::from_le_slice(&amount),
                    calldata,
                });
            }
            Err(_) => {
                return Ok(Burn {
                    burn_id: FixedBytes::from_slice(&burn_id),
                    network,
                    token,
                    amount: U256::from_le_slice(&amount),
                    calldata: None,
                });
            }
        }
    }
}

pub enum CustomTxMsg {
    // OwshenAirdrop {
    //     owshen_address: Address,
    //     owshen_sig: tx::owshen_airdrop::babyjubjub::Signature,
    // },
    MintTx(Mint),
    BurnTx(Burn),
}
impl CustomTxMsg {
    pub fn as_rlp(&self) -> Vec<u8> {
        match self {
            // CustomTxMsg::OwshenAirdrop {
            //     owshen_address,
            //     owshen_sig,
            // } => {
            //     let mut stream = RlpStream::new_list(3);
            //     stream.append(&"owshen-airdrop");
            //     stream.append(&owshen_address.to_vec());
            //     stream.append(&owshen_sig.as_bytes().to_vec());
            //     stream.out().into()
            // }
            CustomTxMsg::MintTx(mint_data) => rlp::encode(mint_data).into(),
            CustomTxMsg::BurnTx(burn_data) => rlp::encode(burn_data).into(),
        }
    }
    pub fn from_rlp(bytes: &[u8]) -> Result<CustomTxMsg> {
        let rlp = rlp::Rlp::new(bytes);
        let tx_type: String = rlp.val_at(0)?;
        match tx_type.as_str() {
            // "owshen-airdrop" => {
            //     let _owshen_address: Vec<u8> = rlp.val_at(1)?;
            //     let _owshen_sig: Vec<u8> = rlp.val_at(2)?;
            //     Ok(CustomTxMsg::OwshenAirdrop {
            //         owshen_address: Address::from_slice(_owshen_address.as_slice()),
            //         owshen_sig: alloy::primitives::Signature::try_from(_owshen_sig.as_slice())?,
            //     })
            // }
            "mint" => Ok(CustomTxMsg::MintTx(rlp::decode(bytes)?)),
            "burn" => Ok(CustomTxMsg::BurnTx(rlp::decode(bytes)?)),
            _ => Err(anyhow!("Invalid tx!")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomTx {
    pub chain_id: u64,
    pub msg: Vec<u8>,
    pub sig: alloy::primitives::Signature,
}

impl CustomTx {
    pub fn msg(&self) -> Result<CustomTxMsg> {
        CustomTxMsg::from_rlp(&self.msg)
    }
    pub async fn create<S: Signer + Sync>(
        signer: &mut S,
        chain_id: u64,
        msg: CustomTxMsg,
    ) -> Result<OwshenTransaction> {
        let signer = signer.with_chain_id(Some(chain_id));
        let msg_rlp = msg.as_rlp();
        let sig = signer.sign_message(&msg_rlp).await?;
        let tx = CustomTx {
            msg: msg_rlp,
            chain_id,
            sig,
        };
        Ok(OwshenTransaction::Custom(tx))
    }
    pub fn signer(&self) -> Result<Address> {
        Ok(self.sig.recover_address_from_msg(&self.msg)?)
    }
}
