// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/OwshenV2.sol";
import "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract UpgradeOwshen is Script {
    function run() external returns (address) {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        address proxy = vm.envAddress("PROXY_ADDRESS");
        address newImpl = address(new OwshenV2());
        UnsafeUpgrades.upgradeProxy(proxy, newImpl, abi.encodeWithSignature("initializeV2(address)", deployer));
        address newImplAddress = Upgrades.getImplementationAddress(proxy);

        vm.stopBroadcast();

        return (newImplAddress);
    }
}
