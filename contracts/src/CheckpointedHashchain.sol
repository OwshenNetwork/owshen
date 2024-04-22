// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./IPoseidon.sol";

contract CheckpointedHashchain {
    IPoseidon2 hasher;
    uint256 last_commitment;
    uint256 checkpoint;
    uint256 head;
    uint256 idx;

    uint256 constant CHECKPOINT_INTERVAL = 1024;

    mapping(uint256 => uint256) public headCommitmentsHistory;
    uint32 public constant HEAD_HISTORY_SIZE = 30;
    uint32 public currentHeadIndex = 0;

    mapping(uint256 => uint256) public checkpointCommitmentsHistory;
    uint32 public constant CHECKPOINT_HISTORY_SIZE = 30;
    uint32 public currentCheckpointIndex = 0;

    constructor(IPoseidon2 _hasher, uint256 _genesis_root) {
        hasher = _hasher;
        last_commitment = 0;
        checkpoint = _genesis_root;
        head = 0;
        idx = 0;
    }

    function set(uint256 _commitment) public {
        last_commitment = _commitment;
        head = head == 0 ? _commitment : hasher.poseidon([head, _commitment]);
        idx += 1;

        uint32 newHeadIndex = (currentHeadIndex + 1) % HEAD_HISTORY_SIZE;
        currentHeadIndex = newHeadIndex;
        headCommitmentsHistory[newHeadIndex] = head;

        if (idx % CHECKPOINT_INTERVAL == 0) {
            checkpoint = checkpoint == 0 ? head : hasher.poseidon([checkpoint, head]);

            uint32 newCheckpointIndex = (currentCheckpointIndex + 1) % CHECKPOINT_HISTORY_SIZE;
            currentCheckpointIndex = newCheckpointIndex;
            checkpointCommitmentsHistory[newCheckpointIndex] = checkpoint;

            head = 0;
        }
    }

    function is_known_checkpoint_head(uint256 _checkpoint) public view returns (bool) {
        if (_checkpoint == 0) {
            return false;
        }

        for (uint32 i = 0; i < CHECKPOINT_HISTORY_SIZE; i++) {
            if (checkpointCommitmentsHistory[i] == _checkpoint) {
                return true;
            }
        }

        return false;
    }

    function is_known_latest_value_head(uint256 _head) public view returns (bool) {
        if (_head == 0) {
            return false;
        }

        for (uint32 i = 0; i < HEAD_HISTORY_SIZE; i++) {
            if (headCommitmentsHistory[i] == _head) {
                return true;
            }
        }

        return false;
    }

    function getLastCommitment() public view returns (uint256) {
        return last_commitment;
    }

    function getCheckpointInterval() public pure returns (uint256) {
        return CHECKPOINT_INTERVAL;
    }
}
