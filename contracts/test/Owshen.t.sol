// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/Owshen.sol";

contract OwshenTest is Test {
    Owshen public owshen;

    function setUp() public {
        address cont;
        owshen = new Owshen(IHasher(cont),0);
    }

    function testDeposit() public {
        owshen.deposit(
            Owshen.Point({x: 123, y: 234}),
            Owshen.Point({x: 123, y: 234}),
            0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6,
            1000
        );
        assertEq(owshen.depositIndex(), 1);
        owshen.deposit(
            Owshen.Point({x: 234, y: 345}),
            Owshen.Point({x: 123, y: 234}),
            0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6,
            2000 // Add the amount here
        );
        assertEq(owshen.depositIndex(), 2);
    }
}
