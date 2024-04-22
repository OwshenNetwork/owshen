use std::{path::PathBuf, str::FromStr, sync::Arc};

use bindings::dive_token::{DiveToken, PrivateProofOfBurn};

use ethers::{
    abi::AbiEncode,
    core::k256::elliptic_curve::SecretKey,
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::{Signer, Wallet as wallet},
    types::Bytes,
    utils::{keccak256, rlp},
};

use structopt::StructOpt;

use crate::{
    config::Wallet,
    fp::Fp,
    hash::hash2,
    helper::proof_to_groth16_proof,
    proof::{mpt_last_prove, mpt_path_prove, Proof},
};

#[derive(StructOpt, Debug)]
pub struct MintOpt {
    #[structopt(long)]
    priv_src: String,
    #[structopt(long)]
    endpoint: String,
    #[structopt(long)]
    chain_id: u64,
    #[structopt(long)]
    token_address: String,
    #[structopt(long)]
    burn_address: String,
    #[structopt(long)]
    encrypted: bool,
    #[structopt(long)]
    mpt_last_zkey_path: String,
    #[structopt(long)]
    mpt_last_witness_path: String,
    #[structopt(long)]
    mpt_path_zkey_path: String,
    #[structopt(long)]
    mpt_path_witness_path: String,
    #[structopt(long)]
    prover_path: String,
}

pub async fn mint(_opt: MintOpt, wallet_path: PathBuf) {
    let provider = Provider::<Http>::try_from(_opt.endpoint.clone()).unwrap_or_else(|e| {
        panic!("Error: failed to create provider: {:?}", e);
    });
    let owshen_wallet = std::fs::read_to_string(&wallet_path)
        .map(|s| {
            let w: Wallet = serde_json::from_str(&s).expect("Invalid wallet file!");
            w
        })
        .ok();

    let mut owshen_wallet = owshen_wallet.unwrap_or_else(|| {
        panic!("Wallet is not initialized!");
    });

    let private_key_bytes = hex::decode(&_opt.priv_src).expect("Invalid hex string for from");
    let private_key: SecretKey<_> =
        SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    let wallet = wallet::from(private_key).with_chain_id(_opt.chain_id);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let eth_account = _opt.priv_src.parse::<LocalWallet>().unwrap_or_else(|e| {
        panic!("Error: failed to parse private key: {:?}", e);
    });

    let token_h160_address = H160::from_str(&_opt.token_address).unwrap_or_else(|e| {
        panic!("Error: failed to parse token address: {:?}", e);
    });

    let contract = DiveToken::new(token_h160_address, client.clone());

    let burn_address = owshen_wallet.get_burn_address_info_by_address(
        _opt.burn_address.parse().unwrap_or_else(|e| {
            panic!("Error: failed to parse burn address: {:?}", e);
        }),
    );
    if burn_address.is_none() {
        panic!("Burn address not found!");
    }
    let h160_burn_address = burn_address.unwrap().address;
    let burn_address_preimage = burn_address.unwrap().preimage;
    let block_number = client.get_block_number().await.unwrap_or_else(|e| {
        panic!("Error: failed to get block number: {:?}", e);
    });
    let block = client
        .get_block(block_number)
        .await
        .unwrap_or_else(|e| {
            panic!("Error: failed to get block: {:?}", e);
        })
        .unwrap_or_else(|| {
            panic!("Block not found!");
        });
    let block_id = BlockId::from(block_number);
    let proof = client
        .get_proof(h160_burn_address, vec![], Some(block_id))
        .await
        .unwrap_or_else(|e| {
            panic!("Error: failed to get proof: {:?}", e);
        });

    let mut layers: Vec<U256> = vec![];
    let (prefix, commit_top, postfix) = get_block_splited_information(&block);
    let account_rlp = get_account_rlp(proof.clone());
    let last_account_proof = proof.account_proof.last().unwrap();
    let prefix_account_rlp =
        last_account_proof[..(last_account_proof.len() - account_rlp.len())].to_vec();

    let coin = owshen_wallet.derive_burnt_coin(proof.balance, _opt.encrypted);
    if _opt.encrypted {
        owshen_wallet.burnt_coins.push(coin.clone());
    }

    let nullifier = hash2([Fp::try_from(burn_address_preimage).unwrap(), Fp::from(0)]);
    let balance = coin.get_balance();

    let (last_proof, mpt_last_commit_upper) = mpt_last_prove(
        coin.salt,
        coin.encrypted,
        prefix_account_rlp,
        proof.clone(),
        burn_address_preimage.to_string(),
        _opt.mpt_last_zkey_path,
        _opt.mpt_last_witness_path,
        _opt.prover_path.clone(),
    )
    .unwrap_or_else(|e| {
        panic!("Error: failed to generate mpt last proofs: {:?}", e);
    });
    let last_groth16_proof = proof_to_groth16_proof(last_proof);
    layers.push(mpt_last_commit_upper);

    let reverse_proof = proof.account_proof.iter().rev();
    let mut path_proofs: Vec<Proof> = vec![];
    let mut root_proof: Proof = Default::default();
    for (index, level) in reverse_proof.enumerate() {
        if index == proof.account_proof.len() - 1 {
            if keccak256(level) != block.state_root.as_bytes() {
                panic!("Not verified!");
            }
            (root_proof, _) = mpt_path_prove(
                coin.salt,
                level.to_vec(),
                block.state_root.as_bytes().to_vec(),
                true,
                _opt.mpt_path_zkey_path.clone(),
                _opt.mpt_path_witness_path.clone(),
                _opt.prover_path.clone(),
            )
            .unwrap_or_else(|e| {
                panic!("Error: failed to generate mpt path proofs: {:?}", e);
            });
        } else {
            let next_level = proof.account_proof[proof.account_proof.len() - index - 2].to_vec();
            let (mpt_path_proof, mpt_path_upper_commit) = mpt_path_prove(
                coin.salt,
                level.to_vec(),
                next_level,
                false,
                _opt.mpt_path_zkey_path.clone(),
                _opt.mpt_path_witness_path.clone(),
                _opt.prover_path.clone(),
            )
            .unwrap_or_else(|e| {
                panic!("Error: failed to generate mpt path proofs: {:?}", e);
            });
            path_proofs.push(mpt_path_proof);
            layers.push(mpt_path_upper_commit);
        }
    }

    let proof_of_burn = PrivateProofOfBurn {
        block_number: U256::from(block_number.as_u32()), // TODO: as_u32 cause to support only ~9 billions blocks
        coin: balance,
        nullifier: nullifier.try_into().unwrap(),
        root_proof: proof_to_groth16_proof(root_proof),
        last_proof: last_groth16_proof,
        is_encrypted: _opt.encrypted,
        target: eth_account.address(),
        state_root: commit_top,
        layers: layers,
        mid_proofs: path_proofs
            .iter()
            .map(|x| proof_to_groth16_proof(x.clone()))
            .collect(),
        header_prefix: prefix,
        header_postfix: postfix,
    };

    println!("Proof of burn: {:?}", proof_of_burn);
    let function = contract.mint_burnt(proof_of_burn);
    let pending_res = function.send().await;
    match pending_res {
        Ok(pending_res) => {
            pending_res.await.unwrap_or_else(|e| {
                panic!("Error: failed to mint: {:?}", e);
            });
            owshen_wallet.set_used_burn_address(h160_burn_address);
            owshen_wallet.save_wallet(wallet_path).unwrap_or_else(|e| {
                panic!("Error: failed to save wallet: {:?}", e);
            });
        }
        Err(e) => {
            println!("ContractError: failed to mint: {:?}", e);
        }
    }
}

fn get_block_splited_information(block: &Block<H256>) -> (Bytes, Bytes, Bytes) {
    let mut rlp_stream = rlp::RlpStream::new();
    rlp_stream.begin_unbounded_list();
    rlp_stream.append(&block.parent_hash);
    rlp_stream.append(&block.uncles_hash);
    if let Some(author) = block.author {
        rlp_stream.append(&author);
    }
    rlp_stream.append(&block.state_root);
    rlp_stream.append(&block.transactions_root);
    rlp_stream.append(&block.receipts_root);
    if let Some(logs_bloom) = block.logs_bloom {
        rlp_stream.append(&logs_bloom);
    }
    rlp_stream.append(&block.difficulty);
    if let Some(number) = block.number {
        rlp_stream.append(&number);
    }
    rlp_stream.append(&block.gas_limit);
    rlp_stream.append(&block.gas_used);
    rlp_stream.append(&block.timestamp);
    rlp_stream.append(&block.extra_data.clone().to_vec());
    if let Some(mix_hash) = block.mix_hash {
        rlp_stream.append(&mix_hash);
    }
    if let Some(nonce) = block.nonce {
        rlp_stream.append(&nonce);
    }

    let optional_headers = vec![
        "baseFeePerGas",
        "withdrawalsRoot",
        "blobGasUsed",
        "excessBlobGas",
        "parentBeaconBlockRoot",
    ];

    for header in optional_headers {
        if let Some(v) = match header {
            "baseFeePerGas" => block.base_fee_per_gas,
            "blobGasUsed" => block.blob_gas_used,
            "excessBlobGas" => block.excess_blob_gas,
            _ => None,
        } {
            rlp_stream.append(&v);
        }
        if let Some(v) = match header {
            "withdrawalsRoot" => block.withdrawals_root,
            "parentBeaconBlockRoot" => block.parent_beacon_block_root,
            _ => None,
        } {
            rlp_stream.append(&v);
        }
    }
    rlp_stream.finalize_unbounded_list();

    let header = Bytes::from(rlp_stream.out().to_vec());
    let bytes_state_root = Bytes::from_str(&block.state_root.encode_hex()); // TODO: check if it's correct
    let start_idx = find(&header, &bytes_state_root.unwrap());
    let end_idx = start_idx + block.state_root.encode_hex().len();
    let prefix = header[..start_idx].to_vec();
    let postfix = header[end_idx..].to_vec();
    let commit_top = header[start_idx..end_idx].to_vec();

    (prefix.into(), commit_top.into(), postfix.into())
}

fn find(header: &Bytes, state_root: &Bytes) -> usize {
    let mut start_idx = 0;
    for i in 0..header.len() {
        if i + state_root.len() < header.len()
            && header[i..i + state_root.len()].to_vec() == state_root.to_vec()
        {
            start_idx = i;
            break;
        }
    }
    start_idx
}

fn get_account_rlp(proof: EIP1186ProofResponse) -> Bytes {
    let mut rlp_stream = rlp::RlpStream::new();
    rlp_stream.begin_unbounded_list();
    rlp_stream.append(&proof.nonce);
    rlp_stream.append(&proof.balance);
    rlp_stream.append(&proof.storage_hash);
    rlp_stream.append(&proof.code_hash);
    rlp_stream.finalize_unbounded_list();
    Bytes::from(rlp_stream.out().to_vec())
}
