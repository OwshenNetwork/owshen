// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

contract Owshen is Initializable, ReentrancyGuardUpgradeable, UUPSUpgradeable {
    using SafeERC20 for IERC20;

    address private owner;
    mapping(uint256 => bool) public isExecuted;

    modifier onlyOwner() {
        require(owner == msg.sender, "ERROR: Caller is not the owner");
        _;
    }

    event WithdrawExecuted(address indexed to, address indexed token, uint256 id, uint256 amount);

    function initialize(address _owner) public initializer {
        __UUPSUpgradeable_init();
        owner = _owner;
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @notice Withdraws the specified amount of ERC20 tokens to the msg.sender if the signature is valid.
     * @param _signature The signature to verify the withdraw request.
     * @param _tokenAddress The address of the ERC20 token to withdraw.
     * @param _amount The amount of ERC20 tokens to withdraw.
     * @param _id The unique id of the withdrawl.
     */
    function withdrawToken(bytes memory _signature, address _tokenAddress, uint256 _amount, uint256 _id)
        public
        nonReentrant
        onlyProxy
    {
        _processWithdraw(_signature, _tokenAddress, _amount, _id);
        IERC20(_tokenAddress).safeTransfer(msg.sender, _amount);
    }

    /**
     * @notice Withdraws the specified amount of the chains native coin to the msg.sender if the signature is valid.
     * @param _signature The signature to verify the withdraw request.
     * @param _amount The amount of native coin to withdraw.
     * @param _id The unique id of the withdrawl.
     */
    function withdrawNative(bytes memory _signature, uint256 _amount, uint256 _id) public nonReentrant onlyProxy {
        require(address(this).balance >= _amount, "ERROR: insufficient contract balance.");
        _processWithdraw(_signature, address(0), _amount, _id);
        payable(msg.sender).transfer(_amount);
    }

    /**
     * @notice Internal function to process withdrawal requests.
     * @param _signature The signature provided by the owner to authorize the withdrawal.
     * @param _tokenAddress The address of the token to withdraw (use address(0) for native coin of the chain).
     * @param _amount The amount of tokens or native coin to withdraw.
     * @param _id The unique id of the withdrawl.
     */
    function _processWithdraw(bytes memory _signature, address _tokenAddress, uint256 _amount, uint256 _id) internal {
        require(!isExecuted[_id], "ERROR: withdraw already executed.");

        bytes32 hash = keccak256(abi.encode(msg.sender, _tokenAddress, _amount, _id, block.chainid));

        bool isSigValid = SignatureChecker.isValidSignatureNow(owner, hash, _signature);

        require(isSigValid, "ERROR: invalid signature.");

        isExecuted[_id] = true;

        emit WithdrawExecuted(msg.sender, _tokenAddress, _id, _amount);
    }

    /**
     * @notice Allows the current owner to transfer ownership to a new address.
     * @param newOwner The address to transfer ownership to.
     */
    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "ERROR: New owner is the zero address");
        owner = newOwner;
    }

    receive() external payable onlyProxy {}
}
