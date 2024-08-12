// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/Owshen.sol";
import "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract DeployOwshen is Script {
    function run() external returns (address, address) {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        address _proxyAddress = Upgrades.deployUUPSProxy("Owshen.sol", abi.encodeCall(Owshen.initialize, (deployer)));

        address implementationAddress = Upgrades.getImplementationAddress(_proxyAddress);

        vm.stopBroadcast();

        return (implementationAddress, _proxyAddress);
    }
}

// PRIVATE_KEY=1 forge script script/Owshen.s.sol --broadcast --fork-url http://localhost:8545
