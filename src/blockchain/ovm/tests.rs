// use alloy::{
//     network::{EthereumWallet, TransactionBuilder},
//     rpc::types::TransactionRequest,
//     signers::local::PrivateKeySigner,
// };

// use crate::{
//     blockchain::Config,
//     config::{self, CHAIN_ID},
//     db::RamKvStore,
//     genesis::GENESIS,
//     types::OwshenTransaction,
// };

// use super::*;

// #[tokio::test]
// async fn test_contract_storage() {
//     let conf = Config {
//         chain_id: CHAIN_ID,
//         owner: None,
//         genesis: GENESIS.clone(),
//         owshen: config::OWSHEN_CONTRACT,
//     };
//     let mut chain: Owshenchain<RamKvStore> = Owshenchain::new(conf, RamKvStore::new());
//     let signer: PrivateKeySigner = PrivateKeySigner::random();
//     let vitalik = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
//         .parse()
//         .unwrap();
//     let tx = TransactionRequest::default()
//         .with_to(vitalik)
//         .with_deploy_code(vec![0, 0, 0, 0])
//         .with_nonce(1)
//         .with_gas_limit(100)
//         .with_max_fee_per_gas(100)
//         .with_max_priority_fee_per_gas(100)
//         .with_chain_id(CHAIN_ID);
//     let wallet = EthereumWallet::new(signer);
//     let tx = OwshenTransaction::Eth(tx.build(&wallet).await.unwrap());
//     chain.apply_tx(&tx).unwrap();
// }
