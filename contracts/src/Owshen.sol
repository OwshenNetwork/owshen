// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./SparseMerkleTree.sol";
import "./MiMC.sol";
import "./CoinWithdrawVerifier.sol";

contract Owshen {
    event Sent(uint256 indexed pub_key, uint index);

    struct Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }

    CoinWithdrawVerifier coin_withdraw_verifier;

    mapping(uint256 => bool) nullifiers;

    MiMC mimc;
    SparseMerkleTree tree;
    uint256 public deposits;

    constructor() {
        tree = new SparseMerkleTree();
        mimc = new MiMC();
        coin_withdraw_verifier = new CoinWithdrawVerifier();
        deposits = 0;
    }

    function deposit(uint256 pub_key) public payable {
        require(msg.value == 1 ether);
        uint256 id_time = deposits << 64 + block.timestamp;
        uint256 leaf = mimc.hashLeftRight(pub_key, id_time); // Hash with coin-number so that each coin is unique
        tree.set(deposits, leaf);
        emit Sent(pub_key, deposits);
        deposits += 1;
    }

    function spend(uint256 nullifier, Proof calldata proof) internal {
        require(nullifiers[nullifier] == false);
        nullifiers[nullifier] = true;
        require(coin_withdraw_verifier.verifyProof(proof.a, proof.b, proof.c, [tree.root()]));
    }

    function send(uint256 nullifier, Proof calldata proof, uint256 pub_key) public {
        spend(nullifier, proof);
        deposit(pub_key);
    }

    function withdraw(uint256 nullifier, Proof calldata proof) public {
        spend(nullifier, proof);
        payable(msg.sender).transfer(1 ether);
    }
}
