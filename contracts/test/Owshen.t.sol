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
        owshen.deposit{value: 1.0 ether}(Owshen.Point({x: 123, y: 234}), Owshen.Point({x: 123, y: 234}));
        assertEq(owshen.deposits(), 1);
        owshen.deposit{value: 1.0 ether}(Owshen.Point({x: 234, y: 345}), Owshen.Point({x: 123, y: 234}));
        assertEq(owshen.deposits(), 2);
    }
}
