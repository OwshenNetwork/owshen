// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

// TODO: Add license! https://gist.github.com/poma/5adb51d49057d0a0edad2cbd12945ac4

interface IHasher {
    function MiMCSponge(
        uint256 in_xL,
        uint256 in_xR,
        uint256 in_k
    ) external pure returns (uint256 xL, uint256 xR);
}

contract MiMC {
    IHasher public immutable hasher;
    uint256 constant FIELD_SIZE =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    constructor(IHasher _hasher) {
        hasher = _hasher;
    }

    function hashLeftRight(
        uint256 _left,
        uint256 _right
    ) public view returns (uint256) {
        require(_left < FIELD_SIZE, "_left should be inside the field");
        require(_right < FIELD_SIZE, "_right should be inside the field");
        uint256 R = _left;
        uint256 C = 0;
        (R, C) = hasher.MiMCSponge(R, C, 0);
        R = addmod(R, _right, FIELD_SIZE);
        (R, C) = hasher.MiMCSponge(R, C, 0);
        return R;
    }
}
