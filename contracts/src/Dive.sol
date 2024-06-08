// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "forge-std/console.sol";

import "./MptPathVerifier.sol";
import "./MptLastVerifier.sol";
import "./SpendVerifier.sol";

contract DiveToken is ERC20 {
    uint256 constant FIELD_SIZE = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

    struct Groth16Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }

    struct PrivateProofOfBurn {
        uint256 blockNumber;
        uint256 coin;
        uint256 nullifier;
        Groth16Proof rootProof;
        Groth16Proof lastProof;
        bool isEncrypted;
        address target;
        bytes state_root;
        uint256[] layers;
        Groth16Proof[] midProofs;
        bytes header_prefix;
        bytes header_postfix;
    }

    mapping(uint256 => bool) public nullifiers;
    mapping(uint256 => bool) public coins;
    mapping(address => uint256) private _burnt_balances;

    uint256 starting_block;
    mapping(uint256 => uint256) public epoch_totals;
    mapping(uint256 => mapping(address => uint256)) public epochs;

    SpendVerifier spend_verifier = new SpendVerifier();
    MptLastVerifier mpt_last_verifier = new MptLastVerifier();
    MptPathVerifier mpt_middle_verifier = new MptPathVerifier();

    event CoinGenerated(address recipient, uint256 coin);
    event CoinSpent(
        address spender, uint256 coin, uint256 remainingCoin, uint256 withdrawnBalance, address destination
    );

    constructor(uint256 initialSupply) ERC20("Dive", "DIVE") {
        _mint(msg.sender, initialSupply);
        starting_block = block.number;
    }

    function verify_proof(PrivateProofOfBurn calldata proof) internal {
        uint256 is_encrypted = proof.isEncrypted ? 1 : 0;
        require(proof.header_prefix.length == 91, "Burnth: invalid header prefix length");
        require(
            keccak256(abi.encodePacked(proof.header_prefix, proof.state_root, proof.header_postfix))
                == blockhash(proof.blockNumber),
            "Burnth: invalid block hash"
        );
        require(!nullifiers[proof.nullifier], "Burnth: nullifier already used");
        nullifiers[proof.nullifier] = true;
        require(
            mpt_middle_verifier.verifyProof(
                proof.rootProof.a,
                proof.rootProof.b,
                proof.rootProof.c,
                [uint256(bytes32(proof.state_root)) % FIELD_SIZE, proof.layers[proof.layers.length - 1]]
            ),
            "MptRootVerifier: invalid proof"
        );
        for (uint256 i = 0; i < proof.layers.length - 1; i++) {
            require(
                mpt_middle_verifier.verifyProof(
                    proof.midProofs[i].a,
                    proof.midProofs[i].b,
                    proof.midProofs[i].c,
                    [proof.layers[i + 1], proof.layers[i]]
                ),
                "MptMiddleVerifier: invalid proof"
            );
        }
        require(
            mpt_last_verifier.verifyProof(
                proof.lastProof.a,
                proof.lastProof.b,
                proof.lastProof.c,
                [proof.layers[0], proof.coin, proof.nullifier, is_encrypted]
            ),
            "MptLastVerifier: invalid proof"
        );
    }

    function get_burnt_balance(address account) public view returns (uint256) {
        return _burnt_balances[account];
    }

    function mint_burnt(PrivateProofOfBurn calldata proof) public {
        verify_proof(proof);

        if (proof.isEncrypted) {
            coins[proof.coin] = true;
            emit CoinGenerated(proof.target, proof.coin);
        } else {
            _burnt_balances[proof.target] += proof.coin;
        }
    }

    function spend_coin(
        uint256 coin,
        uint256 remainingCoin,
        uint256 withdrawnBalance,
        address destination,
        Groth16Proof calldata proof
    ) external {
        require(coins[coin], "DiveSpend: coin is not valid");
        coins[coin] = false;

        require(
            spend_verifier.verifyProof(proof.a, proof.b, proof.c, [coin, remainingCoin, withdrawnBalance]),
            "SpendVerifier: invalid proof"
        );

        coins[remainingCoin] = true;
        _burnt_balances[destination] += withdrawnBalance;

        emit CoinSpent(msg.sender, coin, remainingCoin, withdrawnBalance, destination);
        emit CoinGenerated(destination, remainingCoin);
    }

    uint256 constant BLOCK_PER_EPOCH = 10;
    uint256 constant MAX_REWARD = 50_000_000_000_000_000_000;
    uint256 constant REWARD_DECREASE_RATE = 10000000000;

    function currentEpoch() public view returns (uint256) {
        return (block.number - starting_block) / BLOCK_PER_EPOCH;
    }

    function approximate(uint256 amount_per_epoch, uint256 num_epochs) public view returns (uint256) {
        uint256 mint_amount = 0;
        uint256 currEpoch = currentEpoch();
        for (uint256 i = 0; i < num_epochs; i++) {
            uint256 epochIndex = currEpoch + i;
            uint256 user = epochs[epochIndex][msg.sender] + amount_per_epoch;
            uint256 total = epoch_totals[epochIndex] + amount_per_epoch;
            mint_amount += (rewardOf(epochIndex) * user) / total;
        }
        return mint_amount;
    }

    function rewardOf(uint256 _epoch) public pure returns (uint256) {
        uint256 reward = MAX_REWARD;
        for (uint256 i = 0; i < _epoch; i++) {
            reward = reward - reward / REWARD_DECREASE_RATE;
        }
        return reward;
    }

    function participate(uint256 amount_per_epoch, uint256 num_epochs) external {
        require(_burnt_balances[msg.sender] >= amount_per_epoch * num_epochs, "Insufficient balance");
        _burnt_balances[msg.sender] -= amount_per_epoch * num_epochs;

        uint256 currEpoch = currentEpoch();
        for (uint256 i = 0; i < num_epochs; i++) {
            epoch_totals[currEpoch + i] += amount_per_epoch;
            epochs[currEpoch + i][msg.sender] += amount_per_epoch;
        }
    }

    function claim(uint256 starting_epoch, uint256 num_epochs) external {
        require(starting_epoch + num_epochs <= currentEpoch(), "Cannot claim an ongoing epoch!");
        uint256 mint_amount = 0;
        for (uint256 i = 0; i < num_epochs; i++) {
            uint256 total = epoch_totals[starting_epoch + i];
            if (total > 0) {
                uint256 user = epochs[starting_epoch + i][msg.sender];
                epochs[i][msg.sender] = 0;
                mint_amount += (rewardOf(starting_epoch + i) * user) / total;
            }
        }
        _mint(msg.sender, mint_amount);
    }
}
