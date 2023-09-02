// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/SparseMerkleTree.sol";

contract SparseMerkleTreeTest is Test {
    SparseMerkleTree public tree;

    function setUp() public {
        tree = new SparseMerkleTree();
    }

    function testTree() public {
        assertEq(tree.root(), uint256(0x1c84fa6e48fb98cd6e53564789999218f732cd6df3d996f8b171f837c141adc3));
        tree.set(123, 234);
        tree.set(345, 456);
        assertEq(tree.root(), uint256(0x2327f352f31bb091a828d10b5ae85b558cf08d85af0b9f8aa296034e5f34f7b7));
    }
}
