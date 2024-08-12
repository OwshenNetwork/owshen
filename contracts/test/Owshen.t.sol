// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";

import {Owshen} from "../src/Owshen.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract SampleOwshenV2 is Owshen {
    function initializeV2() public {}

    function echo() public pure returns (uint256) {
        return 1;
    }
}

contract SampleERC20 is ERC20 {
    constructor() ERC20("SampleERC20", "S") {
        this;
    }

    function mint(address to, uint256 amount) public {
        _mint(to, amount);
    }
}

contract OwshenTest is Test {
    Owshen public yoctoChain;
    address public owner;
    address public _proxyAddress;
    address public _implAddress;
    uint256 internal ownerPrivateKey;
    address public user;
    uint256 internal userPrivateKey;
    SampleERC20 public token;

    error InvalidNumber(uint256 number);

    function setUp() public {
        ownerPrivateKey = 1;
        owner = vm.addr(ownerPrivateKey);
        userPrivateKey = 2;
        user = vm.addr(userPrivateKey);

        vm.startPrank(owner);

        _implAddress = address(new Owshen());
        _proxyAddress =
            UnsafeUpgrades.deployUUPSProxy(_implAddress, abi.encodeWithSignature("initialize(address)", owner));

        token = new SampleERC20();

        vm.stopPrank();
    }

    function depoist(uint256 amount) internal {
        vm.startPrank(user);

        token.mint(user, amount);
        require(token.balanceOf(user) == amount, "balance of user is not equal to amount");

        token.transfer(_proxyAddress, amount);
        require(token.balanceOf(user) == 0, "balance of user should be 0");
        require(token.balanceOf(_proxyAddress) == amount, "balance of yoctoChain should be equal to amount");

        vm.stopPrank();
    }

    function test_recive_success() public {
        require(token.balanceOf(user) == 0, "balance of user should be 0");
        require(token.balanceOf(_proxyAddress) == 0, "balance of yoctoChain should be 0");
        depoist(100);
        require(token.balanceOf(user) == 0, "balance of user should be 0");
        require(token.balanceOf(_proxyAddress) == 100, "balance of yoctoChain should be equal to amount");
    }

    function test_withdraw_token_success() public {
        depoist(100);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(token), 100, 1, block.chainid));

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);
        address signer = ecrecover(hash, v, r, s);
        require(owner == signer, "owner should be signer");

        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawToken(bytes,address,uint256,uint256)", signature, address(token), 100, 1)
        );
        assertTrue(success);

        vm.stopPrank();

        require(token.balanceOf(user) == 100, "balance of user should be 100");
        require(token.balanceOf(_proxyAddress) == 0, "balance of yoctoChain should be 0");
    }

    function test_withdraw_token_fail_invalid_signature() public {
        depoist(100);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(token), 50, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawToken(bytes,address,uint256,uint256)", signature, address(token), 100, 1)
        );

        // call returns success because of revert was expected
        assertTrue(success);

        vm.stopPrank();

        require(token.balanceOf(user) == 0, "balance of user should be 0");
        require(token.balanceOf(_proxyAddress) == 100, "balance of yoctoChain should be 100");
    }

    function test_withdraw_token_fail_invalid_id() public {
        depoist(100);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(token), 100, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        vm.expectRevert();
        yoctoChain.withdrawToken(signature, address(token), 100, 2);

        vm.stopPrank();

        require(token.balanceOf(user) == 0, "balance of user should be 0");
        require(token.balanceOf(_proxyAddress) == 100, "balance of yoctoChain should be 100");
    }

    function test_withdraw_token_fail_signature_replay() public {
        depoist(100);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(token), 100, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        (bool success1,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawToken(bytes,address,uint256,uint256)", signature, address(token), 100, 1)
        );
        assertTrue(success1);

        vm.expectRevert();
        (bool success2,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawToken(bytes,address,uint256,uint256)", signature, address(token), 100, 1)
        );
        assertTrue(success2);

        vm.stopPrank();

        require(token.balanceOf(user) == 100, "balance of user should be 100");
        require(token.balanceOf(_proxyAddress) == 0, "balance of yoctoChain should be 0");
    }

    function test_withdraw_native_success() public {
        vm.deal(_proxyAddress, 100 ether);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(0), 100 ether, 1, block.chainid));

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);
        address signer = ecrecover(hash, v, r, s);
        require(owner == signer, "owner should be signer");

        require(user.balance == 0, "balance of user should be 0 ether");
        require(_proxyAddress.balance == 100 ether, "balance of yoctoChain should be 100 ether");
        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 100 ether, 1)
        );
        assertTrue(success);

        vm.stopPrank();

        require(user.balance == 100 ether, "balance of user should be 100 ether");
        require(_proxyAddress.balance == 0, "balance of yoctoChain should be 0 ether");
    }

    function test_withdraw_native_fail_invalid_signature() public {
        vm.deal(_proxyAddress, 100 ether);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(0), 50, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 100 ether, 1)
        );
        assertTrue(success);

        vm.stopPrank();

        require(user.balance == 0, "balance of user should be 0 ether");
    }

    function test_withdraw_native_fail_invalid_id() public {
        vm.deal(_proxyAddress, 100 ether);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(0), 100, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 100 ether, 2)
        );
        assertTrue(success);

        vm.stopPrank();

        require(user.balance == 0, "balance of user should be 0 ether");
    }

    function test_withdraw_native_fail_invalid_balance() public {
        vm.deal(_proxyAddress, 100 ether);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(0), 100, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 101 ether, 1)
        );
        assertTrue(success);

        vm.stopPrank();

        require(user.balance == 0, "balance of user should be 0 ether");
    }

    function test_withdraw_native_fail_signature_replay() public {
        vm.deal(_proxyAddress, 100 ether);
        vm.startPrank(user);

        bytes32 hash = keccak256(abi.encode(user, address(0), 100, 1, block.chainid));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, hash);
        bytes memory signature = abi.encodePacked(r, s, v);

        (bool success1,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 100, 1)
        );
        assertTrue(success1);

        vm.expectRevert();
        (bool success2,) = address(_proxyAddress).call(
            abi.encodeWithSignature("withdrawNative(bytes,uint256,uint256)", signature, 100, 1)
        );
        assertTrue(success2);

        vm.stopPrank();

        require(user.balance == 100, "balance of user should be 100 ether");
    }

    function test_transferOwnership_success() public {
        vm.startPrank(owner);
        (bool success1,) = address(_proxyAddress).call(abi.encodeWithSignature("transferOwnership(address)", user));
        assertTrue(success1);

        vm.startPrank(user);
        (bool success2,) = address(_proxyAddress).call(abi.encodeWithSignature("transferOwnership(address)", owner));
        assertTrue(success2);

        vm.stopPrank();
    }

    function test_transferOwnership_fail_zero_address() public {
        vm.startPrank(owner);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(abi.encodeWithSignature("transferOwnership(address)", address(0)));
        assertTrue(success);

        vm.stopPrank();
    }

    function test_transferOwnership_fail_not_owner() public {
        vm.startPrank(user);

        vm.expectRevert();
        (bool success,) = address(_proxyAddress).call(abi.encodeWithSignature("transferOwnership(address)", user));
        assertTrue(success);

        vm.stopPrank();
    }

    function test_upgrade_success() public {
        vm.startPrank(owner);

        address newImpl = address(new SampleOwshenV2());
        UnsafeUpgrades.upgradeProxy(_proxyAddress, newImpl, abi.encodeWithSignature("initializeV2()"));
        address newImplAddress = Upgrades.getImplementationAddress(_proxyAddress);

        require(newImplAddress == newImpl, "new implementation address is wrong");
        require(newImplAddress != _implAddress, "new implementation address should be different from old one");

        (bool success, bytes memory data) = address(_proxyAddress).staticcall(abi.encodeWithSignature("echo()"));
        assertTrue(success);
        uint256 result = abi.decode(data, (uint256));
        require(result == 1, "result should be 1");
        vm.stopPrank();
    }
}
