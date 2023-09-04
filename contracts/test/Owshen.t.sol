// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/Owshen.sol";

contract OwshenTest is Test {
    Owshen public counter;

    function setUp() public {
        counter = new Owshen();
        counter.setNumber(0);
    }

    function testDeposit() public {
        counter.deposit{value: 1.0 ether}(123);
        assertEq(counter.deposits(), 1);
        counter.deposit{value: 1.0 ether}(234);
        assertEq(counter.deposits(), 2);
    }

    function testIncrement() public {
        counter.increment();
        assertEq(counter.number(), 1);
    }

    function testSetNumber(uint256 x) public {
        counter.setNumber(x);
        assertEq(counter.number(), x);
    }
}
