// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./CheckpointedHashchain.sol";
import "./IPoseidon.sol";
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

    IPoseidon4 poseidon4;

    CheckpointedHashchain chc;
    uint256 public depositIndex = 0;
    uint256 constant CHECKPOINT_INTERVAL = 1024;

    /**
     * @dev The constructor
     */
    constructor(IPoseidon4 _poseidon4, IPoseidon2 _poseidon2, uint256 _genesis_root, uint256 _deposit_index) {
        poseidon4 = _poseidon4;
        coin_withdraw_verifier = new CoinWithdrawVerifier();
        chc = new CheckpointedHashchain(_poseidon2, _genesis_root);
        depositIndex = _deposit_index;
    }

    function deposit(Point calldata _pub_key, Point calldata _ephemeral, address _tokenAddress, uint256 _amount)
        public
        payable
    {
        uint256 uint_tokenaddress = getUintTokenAddress(_tokenAddress);
        uint256 leaf = poseidon4.poseidon([_pub_key.x, _pub_key.y, _amount, uint_tokenaddress]);
        chc.set(leaf);
        _processDeposit(msg.sender, address(this), _tokenAddress, _amount);
        emit Sent(_ephemeral, depositIndex, block.timestamp, _amount, uint_tokenaddress, leaf);
        depositIndex += 1;
    }

    function _processDeposit(address _from, address _to, address _token, uint256 _amount) internal {
        require(msg.value == 0, "ETH value is supposed to be 0 for ERC20 instance");
        IERC20(_token).transferFrom(_from, _to, _amount);
    }

    function spend(
        Proof calldata _proof,
        uint256 _checkpoint_head,
        uint256 _latest_values_commitment_head,
        uint256[2] calldata _nullifiers,
        uint256[2] memory _commitments
    ) internal {
        require(
            chc.is_known_checkpoint_head(_checkpoint_head) || depositIndex < CHECKPOINT_INTERVAL,
            "Invalid checkpoint head"
        );
        require(
            chc.is_known_latest_value_head(_latest_values_commitment_head) || depositIndex % CHECKPOINT_INTERVAL == 0,
            "Invalid latest values commitment head"
        );

        require(_nullifiers[0] != _nullifiers[1], "Nullifiers needs to be different");

        require(!nullifiers[_nullifiers[0]] || _nullifiers[0] == 0, "Nullifier has been spent");
        nullifiers[_nullifiers[0]] = true;

        require(!nullifiers[_nullifiers[1]] || _nullifiers[1] == 0, "Nullifier has been spent");
        nullifiers[_nullifiers[1]] = true;

        require(
            coin_withdraw_verifier.verifyProof(
                _proof.a,
                _proof.b,
                _proof.c,
                [
                    _checkpoint_head,
                    _latest_values_commitment_head,
                    _nullifiers[0],
                    _nullifiers[1],
                    _commitments[0],
                    _commitments[1]
                ]
            ),
            "Invalid proof"
        );
    }

    function withdraw(
        uint256 _checkpoint_head,
        uint256 _latest_values_commitment_head,
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
        uint256 null_commitment = poseidon4.poseidon([uint256(uint160(_to)), 0, _amount, uint_tokenaddress]);

        spend(_proof, _checkpoint_head, _latest_values_commitment_head, _nullifiers, [null_commitment, _commitment]);
        chc.set(_commitment);

        IERC20 payToken = IERC20(_tokenAddress);
        payToken.transfer(_to, _amount);
        emit Sent(
            _ephemeral, depositIndex, block.timestamp, _obfuscated_remaining_amount, uint_tokenaddress, _commitment
        );
        emit Spend(_nullifiers[0]);
        emit Spend(_nullifiers[1]);
        depositIndex += 1;
    }

    function send(
        uint256 _checkpoint_head,
        uint256 _latest_values_commitment_head,
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
        spend(_proof, _checkpoint_head, _latest_values_commitment_head, _nullifiers, [_commitments[1], _commitments[0]]);
        chc.set(_commitments[1]);
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
            chc.set(_commitments[0]);
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
    function isSpentArray(uint256[] calldata _nullifierHashes) external view returns (bool[] memory spent) {
        spent = new bool[](_nullifierHashes.length);
        for (uint256 i = 0; i < _nullifierHashes.length; i++) {
            if (isSpent(_nullifierHashes[i])) {
                spent[i] = true;
            }
        }
    }

    function head() public view returns (uint256) {
        return chc.getLastCommitment();
    }

    function getUintTokenAddress(address _token_address) private pure returns (uint256) {
        return uint256(uint160(_token_address));
    }
}
