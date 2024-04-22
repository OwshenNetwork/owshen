pragma circom 2.1.5;

include "./utils.circom";
include "./concat.circom";

template Mux() {
    signal input c[2];
    signal input s;
    signal output out;
    out <== c[0] + s * (c[1] - c[0]);
}

template RlpInt(N) {
    signal input num;
    signal output out[N];
    signal output outLen;

    component decomp = ByteDecompose(N);
    decomp.num <== num;

    component length = GetRealByteLength(N);
    length.bytes <== decomp.bytes;

    component reversed = ReverseArray(N);
    reversed.bytes <== decomp.bytes;
    reversed.realByteLen <== length.len;

    component isSingleByte = LessThan(252);
    isSingleByte.in[0] <== num;
    isSingleByte.in[1] <== 128;

    component isZero = IsZero();
    isZero.in <== num;

    outLen <== (1 - isSingleByte.out) + length.len + isZero.out;

    component firstRlpByteSelector = Mux();
    firstRlpByteSelector.c[0] <== 0x80 + length.len;
    firstRlpByteSelector.c[1] <== num;
    firstRlpByteSelector.s <== isSingleByte.out;

    out[0] <== firstRlpByteSelector.out + isZero.out * 0x80;
    for (var i = 1; i < N; i++) {
        out[i] <== (1 - isSingleByte.out) * reversed.out[i-1];
    }
}

template Rlp() {
    signal input nonce;
    signal input balance; 
    signal input storage_hash[32];
    signal input code_hash[32];
    signal output rlp_encoded[99];
    signal output rlp_encoded_len;

    var storageAndCodeHashRlpLen = 66;
    signal storageAndCodeHashRlpEncoded[storageAndCodeHashRlpLen];

    component nonceRlp = RlpInt(10);
    nonceRlp.num <== nonce;
    component balanceRlp = RlpInt(21);
    balanceRlp.num <== balance;

    storageAndCodeHashRlpEncoded[0] <== 0x80 + 32;
    for (var i = 0; i < 32; i++) {
        storageAndCodeHashRlpEncoded[i + 1] <== storage_hash[i]; 
    }
    storageAndCodeHashRlpEncoded[33] <== 0x80 + 32;
    for (var i = 0; i < 32; i++) {
        storageAndCodeHashRlpEncoded[i + 34] <== code_hash[i];
    }

    component concat1 = Concat(10, 21);
    concat1.a <== nonceRlp.out;
    concat1.aLen <== nonceRlp.outLen;
    concat1.b <== balanceRlp.out;
    concat1.bLen <== balanceRlp.outLen;

    component concat2 = Concat(31, 66);
    concat2.a <== concat1.out;
    concat2.aLen <== concat1.outLen;
    concat2.b <== storageAndCodeHashRlpEncoded;
    concat2.bLen <== storageAndCodeHashRlpLen;

    rlp_encoded[0] <== 0xf8; 
    rlp_encoded[1] <== concat2.outLen;
    for(var i = 0; i < 97; i++) {
        rlp_encoded[i+2] <== concat2.out[i];
    }
    rlp_encoded_len <== 2 + concat2.outLen;
}
