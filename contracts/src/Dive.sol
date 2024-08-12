// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "forge-std/console.sol";

contract Dive is ERC20 {
    constructor() ERC20("Dive", "DIVE") {
        _mint(msg.sender, 1_000_000_000 ether);
    }
}
