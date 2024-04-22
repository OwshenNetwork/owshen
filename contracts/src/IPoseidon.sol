// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

// TODO: Add license! https://gist.github.com/poma/5adb51d49057d0a0edad2cbd12945ac4

interface IPoseidon4 {
    function poseidon(uint256[4] memory input) external pure returns (uint256);
}

interface IPoseidon2 {
    function poseidon(uint256[2] memory input) external pure returns (uint256);
}
