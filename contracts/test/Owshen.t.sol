// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/Owshen.sol";

contract OwshenTest is Test {
    Owshen public owshen;

    function setUp() public {
        owshen = new Owshen();
    }

    function testDeposit() public {
        owshen.deposit{value: 1.0 ether}(123, 234, 345, 456);
        assertEq(owshen.deposits(), 1);
        owshen.deposit{value: 1.0 ether}(234, 345, 456, 567);
        assertEq(owshen.deposits(), 2);
    }
}
