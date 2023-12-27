// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./MiMC.sol";

contract SparseMerkleTree {
    IHasher mimc;
    uint256 genesis_root;

    constructor(IHasher _hasher, uint256 _genesis_root) {
        genesis_root = _genesis_root;
        mimc = _hasher;
    }

    mapping(uint256 => mapping(uint256 => uint256)) layers;

    function get_at_layer(uint256 layer, uint256 index) public view returns (uint256) {
        if(layer == 15 && index == 0) {
            return genesis_root;
        }
        uint256 value = layers[layer][index];
        if (value == 0) {
            value = zeros(layer);
        }
        return value;
    }

    function root() public view returns (uint256) {
        return get_at_layer(16, 0);
    }

    function set(uint256 index, uint256 value) public {
        for (uint256 i = 0; i <= 16; i++) {
            layers[i][index] = value;
            uint256 leftmost = index - (index % 4);
            uint256[4] memory vals;
            vals[0] = get_at_layer(i, leftmost);
            vals[1] = get_at_layer(i, leftmost + 1);
            vals[2] = get_at_layer(i, leftmost + 2);
            vals[3] = get_at_layer(i, leftmost + 3);
            vals[index % 4] = value;
            value = mimc.poseidon(vals);
            index /= 4;
        }
    }

    function zeros(uint256 i) public pure returns (uint256) {
        if (i == 0) {
            return uint256(0x0);
        } else if (i == 1) {
            return uint256(0x532fd436e19c70e51209694d9c215250937921b8b79060488c1206db73e9946);
        } else if (i == 2) {
            return uint256(0x1ea8dbbca1ca3a574b1b871aad1e8bc47571c7e11a99a6fb25f3f19d4bfad32c);
        } else if (i == 3) {
            return uint256(0x1b98ec5a992cd3688e655aef8674386f4e84f0a442eaab4840427e359c214dd2);
        } else if (i == 4) {
            return uint256(0xfd83a3939005974d559c13c652a31942ffd61cab9a038e04239af96cafaee14);
        } else if (i == 5) {
            return uint256(0x11eb54967ae25ce42205223461cda9dbb1a1811db38b71e404d1fd94c9627777);
        } else if (i == 6) {
            return uint256(0x1c682a99981cf10a215d2788721e0ca060ddc0932cbb04c903eee3bd899dcfc0);
        } else if (i == 7) {
            return uint256(0x2b3de1ae8e72d64a5cb03934061e3fd03789273f370e32d576ae54b4cbeca719);
        } else if (i == 8) {
            return uint256(0x1b64ed0dc55f80f1f3da256b451baf16110fc268311a13fe903f87945b734861);
        } else if (i == 9) {
            return uint256(0x6efb4c4a55b0225fd1088d357b2432f422b0593015c57980e373ecef9029c72);
        } else if (i == 10) {
            return uint256(0x262138dff44f5352d2d7ec48461c771444a6c00c761a3263735d0a99143a51f1);
        } else if (i == 11) {
            return uint256(0x7ea118d86ace14a8eaa26fc18b36fefe7d5f5b00e0eb9ebfd1c88f37131c178);
        } else if (i == 12) {
            return uint256(0xc2a4af81a2cd78159ce2f3d7ee05367fc5b3c3e1becea2a86d1e66dd27ffb7f);
        } else if (i == 13) {
            return uint256(0x1a953a710c22f79c369c976fc9981ffdbfe236695297fec65886e645e6fa2884);
        } else if (i == 14) {
            return uint256(0x16b0c5e286286eb6c4774ad6292b4f0bdc5b36cb7890b27544268892ae255e0d);
        } else if (i == 15) {
            return uint256(0x2dd7186449cc82702fb8f2d7fa86e0095263f8afb521e1976f499e67a0d7cbb8);
        } else if (i == 16) {
            return uint256(0x151399c724e17408a7a43cdadba2fc000da9339c56e4d49c6cdee6c4356fbc68);
        } else {
            revert("Index out of bounds");
        }
    }
}
