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
        Point ephemeral,
        uint256 index,
        uint256 timestamp,
        uint256 hint_amount,
        uint256 hint_tokenAddress,
        uint256 commitment
    );

    event Spend(uint256 nullifier);
    event Deposit(Point indexed pub_key, Point ephemeral, uint256 nullifier);

    CoinWithdrawVerifier coin_withdraw_verifier;
    mapping(uint256 => bool) nullifiers;

    IHasher mimc;
    SparseMerkleTree tree;
    uint256 public depositIndex = 4 ** 15;

    /**
     * @dev The constructor
     */
    constructor(IHasher _hasher, uint256 _genesis_root) {
        tree = new SparseMerkleTree(_hasher, _genesis_root);
        mimc = _hasher;
        coin_withdraw_verifier = new CoinWithdrawVerifier();
    }

    function deposit(
        Point calldata _pub_key,
        Point calldata _ephemeral,
        address _tokenAddress,
        uint256 _amount
    ) public payable {
        uint256 uint_tokenaddress = getUintTokenAddress(_tokenAddress);
        uint256 leaf = mimc.poseidon(
            [_pub_key.x, _pub_key.y, _amount, uint_tokenaddress]
        );
        tree.set(depositIndex, leaf);
        _processDeposit(msg.sender, address(this), _tokenAddress, _amount);
        emit Sent(
            _ephemeral,
            depositIndex,
            block.timestamp,
            _amount,
            uint_tokenaddress,
            leaf
        );
        depositIndex += 1;
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

    function spend(
        Proof calldata proof,
        uint256 _root,
        uint256[2] calldata _nullifiers,
        uint256[2] memory _commitments
    ) internal {
        require(tree.is_known_root(_root), "Invalid root");

        require(_nullifiers[0] != _nullifiers[1], "Nullifiers needs to be different");

        require(!nullifiers[_nullifiers[0]] || _nullifiers[0] == 0, "Nullifier has been spent");
        nullifiers[_nullifiers[0]] = true;

        require(!nullifiers[_nullifiers[1]] || _nullifiers[1] == 0, "Nullifier has been spent");
        nullifiers[_nullifiers[1]] = true;

        require(
            coin_withdraw_verifier.verifyProof(
                proof.a,
                proof.b,
                proof.c,
                [_root, _nullifiers[0], _nullifiers[1], _commitments[0], _commitments[1]]
            ),
            "Invalid proof"
        );

    }

    function withdraw(
        uint256 _root,
        Proof calldata _proof,
        Point calldata _ephemeral,
        uint256[2] calldata _nullifiers,
        address _tokenAddress,
        uint256 _amount,
        uint256 _obfuscated_remaining_amount,
        address _to,
        uint256 _commitment
    ) public {
        uint256 uint_tokenaddress = getUintTokenAddress(_tokenAddress);
        uint256 null_commitment = mimc.poseidon([uint256(uint160(_to)), 0, _amount, uint_tokenaddress]);
        spend(_proof, _root, _nullifiers, [null_commitment, _commitment]);
        tree.set(depositIndex, _commitment);
        IERC20 payToken = IERC20(_tokenAddress);
        payToken.transfer(_to, _amount);
        emit Sent(
            _ephemeral,
            depositIndex,
            block.timestamp,
            _obfuscated_remaining_amount,
            uint_tokenaddress,
            _commitment
        );
        emit Spend(_nullifiers[0]);
        emit Spend(_nullifiers[1]);
        depositIndex += 1;
    }

    function send(
        uint256 _root,
        Proof calldata _proof,
        Point calldata _receiver_ephemeral,
        Point calldata _sender_ephemeral,
        uint256[2] calldata _nullifiers,
        uint256[2] calldata _commitments,
        uint256 _receiver_token_address_hint,
        uint256 _sender_token_address_hint,
        uint256 _receiver_amount_hint,
        uint256 _sender_amount_hint,
        bool _is_dual_output
    ) public {
        spend(_proof, _root, _nullifiers, [_commitments[1], _commitments[0]]);
        tree.set(depositIndex, _commitments[1]);
        emit Sent(
            _receiver_ephemeral,
            depositIndex,
            block.timestamp,
            _receiver_amount_hint,
            _receiver_token_address_hint,
            _commitments[1]
        );
        depositIndex += 1;
        if (_is_dual_output) {
            tree.set(depositIndex, _commitments[0]);
            emit Sent(
                _sender_ephemeral,
                depositIndex,
                block.timestamp,
                _sender_amount_hint,
                _sender_token_address_hint,
                _commitments[0]
            );
            depositIndex += 1;
        }
        emit Spend(_nullifiers[0]);
        emit Spend(_nullifiers[1]);
    }

    /**
     * @dev whether a nullifier is already spent
     */
    function isSpent(uint256 _nullifierHash) public view returns (bool) {
        return nullifiers[_nullifierHash];
    }

    /**
     * @dev whether an array of nullifiers is already spent
     */
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
