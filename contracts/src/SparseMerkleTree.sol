// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./MiMC.sol";

contract SparseMerkleTree {
    MiMC mimc;

    constructor(IHasher _hasher) {
        mimc = new MiMC(_hasher);
    }

    mapping(uint256 => mapping(uint256 => uint256)) layers;

    function get_at_layer(uint256 layer, uint256 index) public view returns (uint256) {
        uint256 value = layers[layer][index];
        if (value == 0) {
            value = zeros(layer);
        }
        return value;
    }

    function root() public view returns (uint256) {
        return get_at_layer(32, 0);
    }

    function set(uint256 index, uint256 value) public {
        for (uint256 i = 0; i < 33; i++) {
            layers[i][index] = value;
            if (index % 2 == 0) {
                value = mimc.hashLeftRight(value, get_at_layer(i, index + 1));
            } else {
                value = mimc.hashLeftRight(get_at_layer(i, index - 1), value);
            }
            index /= 2;
        }
    }

    function zeros(uint256 i) public pure returns (uint256) {
        if (i == 0) return uint256(0x0000000000000000000000000000000000000000000000000000000000000000);
        else if (i == 1) return uint256(0x2d9fea8398a61ea1997e7d748364c0fdb49412c4dbabc1578375ade642e85581);
        else if (i == 2) return uint256(0x1234a304a6250851669d511fd01a93eef2fd88d84bbb8b089021393bd6314ace);
        else if (i == 3) return uint256(0x11a759c3e46852e6ee14e3bb8f7158c62d9270217563f56726b3d5ae719e77cf);
        else if (i == 4) return uint256(0x2802b08e40189aad1966fe84660e04b0a92cd6a0c6a8845915244bb888d60cc1);
        else if (i == 5) return uint256(0x278861ed6103a39717d415bec985d336cca450c01e5e2782c33949ba10b986a5);
        else if (i == 6) return uint256(0x020a474de93592d5b1127589ba705a0f0016ab559a799b0c7ce76429b3243b0a);
        else if (i == 7) return uint256(0x2116864224ac0352d9637a12017a5a9c87417becc8147bd0d7654d2c66ea25bf);
        else if (i == 8) return uint256(0x2343742077c09474f0309521118da4d25cc0b62c59ace9bb68872de00a6eabad);
        else if (i == 9) return uint256(0x2466b9845a16b0ebf5c7186e91005508b795731630fed543c3283ec5d6979d4d);
        else if (i == 10) return uint256(0x1a7857e456c4c61a08577945811e341c6aea2e9ffbc067ae2c6ba84e234274d8);
        else if (i == 11) return uint256(0x117f1149dee533f1fd19526b414b2d2bef7a58bf40700d9f2f20a48110245caf);
        else if (i == 12) return uint256(0x1f65c6939e8ea9cf0721305bdcb4d46f110e47d36cb7425c60548ed3ffd6dec4);
        else if (i == 13) return uint256(0x2a127272f233f9414c4db2bfb72da605681103e489c5c27344b3e3e05c9731d6);
        else if (i == 14) return uint256(0x1597270471e05f72ac53719b6fe4cb6ce6322a730706c34da70a98e6928f311f);
        else if (i == 15) return uint256(0x144e4dcfca8d432a7bf23d6c57ef2cc86f371ea4b32527388ea4ce26ecb0a8e8);
        else if (i == 16) return uint256(0x1a781c1159b0f76ac76b5d8fe1ddf457f75d0033fef4d6f44f2c7787825c3229);
        else if (i == 17) return uint256(0x16fb3e5ac86d9a09fc73706c4c778707cdb4e6fd15b7cbd8e83519b85968b13d);
        else if (i == 18) return uint256(0x2fc35be02fb43a8c4d17b79f104b5b53f4ade39d020702a96ba45af57a747ad4);
        else if (i == 19) return uint256(0x17a97f2fd44b04668cb9d53a7cd3ddcf4fa2f87e0eba8e080180f5848646363d);
        else if (i == 20) return uint256(0x231118223cad627f42312b09cc5c1d971028532ba718f4804ade66b62d69d0d8);
        else if (i == 21) return uint256(0x2d51a6c6968f7f9937b0ce246700a7ef2642d602704affe3f6826099ae89c314);
        else if (i == 22) return uint256(0x040c403e552a7da3347a814cfa68f7179022f4bad093f5d51c36f274e6c00216);
        else if (i == 23) return uint256(0x2b51a6f81676a81df506994d2a2fe42221636a7be5316dda9e72a4cfb4b6713e);
        else if (i == 24) return uint256(0x09bac36c373cabdce29c71d6b6ff3bce8f23697cc8fc1bf773764e8c6c87106a);
        else if (i == 25) return uint256(0x1ac61b00797e2af34f53e095ba779bfe5585f1028e99b93853fd7f6ac2695ec6);
        else if (i == 26) return uint256(0x14126628e2921bf57d06edeebe4b4d765b78098face56dd6c3ff3863fa8d4e14);
        else if (i == 27) return uint256(0x10b85caa5c83624cbbf38bc8e4e50fd7ccae13bfc56cae2fe5aef4b63f5d213a);
        else if (i == 28) return uint256(0x2e8383359c191e60d2d64cee201f80417bfd23e25e8b459f38eb0dcef5d27d3e);
        else if (i == 29) return uint256(0x282dc9be0b4e379381b409259502806071421b72e80ed43b74f6771a2c7588fd);
        else if (i == 30) return uint256(0x03b92d4561fbbafc65869388f1ca7983d1ea6786cd92d36f1d9cc615c4e90e5e);
        else if (i == 31) return uint256(0x225e55edbfb735fa5dc7b2253e85fa2d785f4d501bf739c1e01cd9945a614142);
        else if (i == 32) return uint256(0x1c84fa6e48fb98cd6e53564789999218f732cd6df3d996f8b171f837c141adc3);
        else revert("Index out of bounds");
    }
}
