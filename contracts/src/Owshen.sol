// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./SparseMerkleTree.sol";
import "./MiMC.sol";
import "./CoinWithdrawVerifier.sol";

contract Owshen {
    event Sent(Point pub_key, Point ephemeral, uint256 index);

    struct Point {
        uint256 x;
        uint256 y;
    }

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

    function deposit(Point calldata pub_key, Point calldata ephemeral) public payable {
        require(msg.value == 1 ether);
        uint256 pub_key_hash = mimc.hashLeftRight(pub_key.x, pub_key.y);
        uint256 leaf = mimc.hashLeftRight(pub_key_hash, block.timestamp);
        tree.set(deposits, leaf);
        emit Sent(pub_key, ephemeral, deposits);
        deposits += 1;
    }

    function spend(uint256 nullifier, Proof calldata proof) internal {
        require(nullifiers[nullifier] == false);
        nullifiers[nullifier] = true;
        require(coin_withdraw_verifier.verifyProof(proof.a, proof.b, proof.c, [tree.root(), nullifier]));
    }

    function send(uint256 nullifier, Proof calldata proof, Point calldata pub_key, Point calldata ephemeral) public {
        spend(nullifier, proof);
        deposit(pub_key, ephemeral);
    }

    function withdraw(uint256 nullifier, Proof calldata proof) public {
        spend(nullifier, proof);
        payable(msg.sender).transfer(1 ether);
    }
}
