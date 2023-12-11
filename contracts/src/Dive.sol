// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract DiveToken is ERC20 {
    constructor(uint256 initialSupply) ERC20("Dive", "DIVE") {
        _mint(msg.sender, initialSupply);
    }
}
