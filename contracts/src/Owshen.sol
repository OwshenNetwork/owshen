// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./SparseMerkleTree.sol";
import "./MiMC.sol";
import "./CoinWithdrawVerifier.sol";
import "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";

contract Owshen {
    using Strings for uint256;
    struct Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }

    struct Point {
        uint256 x;
        uint256 y;
    }

    event Sent(
        Point pub_key,
        Point ephemeral,
        uint256 index,
        uint256 timestamp,
        uint256 _amount,
        address _tokenAddress,
        uint256 _leaf,
        uint256 _unit_token_address
    );

    event Withdraw(uint256 nullifier);
    event Deposit(Point indexed pub_key, Point ephemeral, uint256 nullifier);

    CoinWithdrawVerifier coin_withdraw_verifier;
    mapping(uint256 => bool) nullifiers;

    MiMC mimc;
    SparseMerkleTree tree;
    uint256 public depositIndex = 0;

    /**
     @dev The constructor
    */
    constructor(IHasher _hasher) {
        tree = new SparseMerkleTree(_hasher);
        mimc = new MiMC(_hasher);
        coin_withdraw_verifier = new CoinWithdrawVerifier();
    }

    function deposit(
        Point calldata _pub_key,
        Point calldata ephemeral,
        address _tokenAddress,
        uint256 _amount,
        address _from,
        address _to
    ) public payable {
        uint256 hash1 = mimc.hashLeftRight(_pub_key.x, _pub_key.y);
        uint256 uint_tokenaddress = getUintTokenAddress(_tokenAddress);
        uint256 hash2 = mimc.hashLeftRight(_amount, uint_tokenaddress);
        uint256 leaf = mimc.hashLeftRight(hash1, hash2);
        tree.set(depositIndex, leaf);
        _processDeposit(_from, _to, _tokenAddress, _amount);
        emit Sent(
            _pub_key,
            ephemeral,
            depositIndex,
            block.timestamp,
            _amount,
            _tokenAddress,
            leaf,
            uint_tokenaddress
        );
        depositIndex += 1;
    }

    function getPointKey(Point memory _pub_key) public pure returns (bytes32) {
        string memory keyString = string(
            abi.encodePacked(_pub_key.x.toString(), ",", _pub_key.y.toString())
        );
        return keccak256(abi.encodePacked(keyString));
    }

    function _processDeposit(
        address _from,
        address _to,
        address _token,
        uint256 _amount
    ) internal {
        require(
            msg.value == 0,
            "ETH value is supposed to be 0 for ERC20 instance"
        );
        IERC20(_token).transferFrom(_from, _to, _amount);
    }

    function spend(uint256 nullifier, Proof calldata proof) internal {
        require(!nullifiers[nullifier], "Nullifier has been spent");
        nullifiers[nullifier] = true;
        require(
            coin_withdraw_verifier.verifyProof(
                proof.a,
                proof.b,
                proof.c,
                [root(), nullifier]
            ),
            "Invalid proof"
        );
    }

    function withdraw(
        uint256 nullifier,
        Proof calldata proof,
        address _tokenAddress,
        uint256 _amount,
        address _to
    ) public {
        spend(nullifier, proof);
        IERC20 payToken = IERC20(_tokenAddress);
        payToken.transfer(_to, _amount);
        emit Withdraw(nullifier);
    }

    /** @dev whether a nullifier is already spent */
    function isSpent(uint256 _nullifierHash) public view returns (bool) {
        return nullifiers[_nullifierHash];
    }

    /** @dev whether an array of nullifiers is already spent */
    function isSpentArray(
        uint256[] calldata _nullifierHashes
    ) external view returns (bool[] memory spent) {
        spent = new bool[](_nullifierHashes.length);
        for (uint256 i = 0; i < _nullifierHashes.length; i++) {
            if (isSpent(_nullifierHashes[i])) {
                spent[i] = true;
            }
        }
    }

    function root() public view returns (uint256) {
        return tree.root();
    }

    function getUintTokenAddress(
        address _token_address
    ) private pure returns (uint256) {
        return uint256(uint160(_token_address));
    }
}
