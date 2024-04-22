pragma circom 2.1.5;

include "./utils/concat.circom";
include "./utils/hasher.circom";
include "./utils/keccak/keccak.circom";
include "./utils/hashbytes.circom";
include "./utils/rlp.circom";
include "./utils/poseidon.circom";


template HashAddress() {
    signal input address[20];
    signal output hash_address[32];

    component addr_decomp[20];
    signal hashed_address_bits[32 * 8];
    signal keccak_input[136 * 8];
    for(var i = 0; i < 136; i++) {
        if(i < 20) {
            addr_decomp[i] = BitDecompose(8);
            addr_decomp[i].num <== address[i];
            for(var j = 0; j < 8; j++) {
                keccak_input[8 * i + j] <== addr_decomp[i].bits[j];
            }
        } else {
            if(i == 20) {
                for(var j = 0; j < 8; j++) {
                    keccak_input[8 * i + j] <== (0x01 >> j) & 1;
                }
            } else if(i == 135) {
                for(var j = 0; j < 8; j++) {
                    keccak_input[8 * i + j] <== (0x80 >> j) & 1;
                }
            } else {
                for(var j = 0; j < 8; j++) {
                    keccak_input[8 * i + j] <== 0;
                }
            }
        }
    }
    component keccak = Keccak(1);
    keccak.in <== keccak_input;
    keccak.blocks <== 1;
    hashed_address_bits <== keccak.out;
    for(var i = 0; i < 32; i++) {
        var sum = 0;
        for(var j = 0; j < 8; j++) {
            sum += (2 ** j) * hashed_address_bits[i * 8 + j];
        }
        hash_address[i] <== sum;
    }
}

template MptLast(maxBlocks, maxLowerLen, security) {

    var maxPrefixLen = maxBlocks * 136 - maxLowerLen;

    signal input burn_preimage;
    component burn_hasher = Poseidon(2);
    burn_hasher.inputs[0] <== burn_preimage;
    burn_hasher.inputs[1] <== burn_preimage;
    component burn_bits = Num2Bits_strict();
    burn_bits.in <== burn_hasher.out;

    signal address[20];
    component byter[20];
    for(var i = 0; i < 20; i++) {
        byter[i] = Bits2Num(8);
        for(var j = 0; j < 8; j++) {
            byter[i].in[j] <== burn_bits.out[8*i+j];
        }
        address[i] <== byter[i].out;
    }
    
    signal input lowerLayerPrefixLen;
    signal input lowerLayerPrefix[maxPrefixLen];

    signal input nonce;
    signal input balance;
    signal input storageHash[32];
    signal input codeHash[32];

    signal input salt;
    signal input encrypted;

    signal output commitUpper;
    signal output encryptedBalance;
    signal output nullifier;

    component nullifier_calc = Poseidon(2);
    nullifier_calc.inputs[0] <== burn_preimage;
    nullifier_calc.inputs[1] <== 0;
    nullifier <== nullifier_calc.out;

    encrypted * (1 - encrypted) === 0;

    component balanceEnc = Poseidon(2);
    balanceEnc.inputs[0] <== balance;
    balanceEnc.inputs[1] <== salt;
    encryptedBalance <== (balanceEnc.out - balance) * encrypted + balance;

    component account_rlp_calculator = Rlp();
    account_rlp_calculator.nonce <== nonce;
    account_rlp_calculator.balance <== balance;
    account_rlp_calculator.storage_hash <== storageHash;
    account_rlp_calculator.code_hash <== codeHash;

    signal lowerLayerLen;
    signal lowerLayer[maxLowerLen];

    lowerLayerLen <== account_rlp_calculator.rlp_encoded_len;
    lowerLayer <== account_rlp_calculator.rlp_encoded;

    signal hash_address[32];
    component addr_hasher = HashAddress();
    addr_hasher.address <== address;
    hash_address <== addr_hasher.hash_address;

    signal expected_prefix[security + 2];
    for(var i = 0; i < security; i++) {
        expected_prefix[i] <== hash_address[32 - security + i];
    }
    expected_prefix[security] <== 1 + 0x80 + 55;
    expected_prefix[security + 1] <== lowerLayerLen;

    signal upperLayerBytes[maxPrefixLen + maxLowerLen];
    signal upperLayerBytesLen;

    component concat = Concat(maxPrefixLen, maxLowerLen);
    concat.a <== lowerLayerPrefix;
    concat.aLen <== lowerLayerPrefixLen;
    concat.b <== lowerLayer;
    concat.bLen <== lowerLayerLen;
    upperLayerBytes <== concat.out;
    upperLayerBytesLen <== concat.outLen;

    signal upperLayer[maxBlocks * 136 * 8];
    signal upperLayerLen;
    upperLayerLen <== 8 * upperLayerBytesLen;
    component decomps[maxPrefixLen + maxLowerLen];
    for(var i = 0; i < maxBlocks * 136; i++) {
        if(i < maxPrefixLen + maxLowerLen) {
            decomps[i] = BitDecompose(8);
            decomps[i].num <== upperLayerBytes[i];
            for(var j = 0; j < 8; j++) {
                upperLayer[8*i + j] <== decomps[i].bits[j];
            }
        } else {
            for(var j = 0; j < 8; j++) {
                upperLayer[8*i + j] <== 0;
            }
        }
    }

    // Commit to upperLayer
    component hasherUpper = HashBytes(maxBlocks * 136, 31);
    hasherUpper.inp <== upperLayerBytes;
    component commitUpperToBlocks = Hasher();
    commitUpperToBlocks.left <== hasherUpper.out;
    commitUpperToBlocks.right <== upperLayerBytesLen;
    component commitUpperToSalt = Hasher();
    commitUpperToSalt.left <== commitUpperToBlocks.hash;
    commitUpperToSalt.right <== salt;
    commitUpper <== commitUpperToSalt.hash;
 }

 component main {public [encrypted]} = MptLast(4, 99, 20);