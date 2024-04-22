pragma circom 2.1.5;

include "./utils.circom";

template Divide() {
    signal input a;
    signal input b;
    signal output out;
    signal output rem;

    out <-- a \ b;
    rem <-- a % b;
    out * b + rem === a;
}

template Padding (nBlocks, blockSize) {
    var N = nBlocks * blockSize;

    signal input a[N];
    signal input aLen;
    signal output out[N];
    signal output num_blocks;

    component div = Divide();
    div.a <== aLen;
    div.b <== blockSize;
    num_blocks <== div.out + 1;

    component is_eqs[N];
    component is_eq_lasts[N];
    signal isEqResults[N + 1];
    isEqResults[0] <== 0;
    for(var i = 0; i < N; i++) {
        is_eqs[i] = IsEqual();
        is_eqs[i].in[0] <== i;
        is_eqs[i].in[1] <== aLen;
        isEqResults[i+1] <== isEqResults[i] + is_eqs[i].out - (isEqResults[i] * is_eqs[i].out);

        is_eq_lasts[i] = IsEqual();
        is_eq_lasts[i].in[0] <== i;
        is_eq_lasts[i].in[1] <== num_blocks * blockSize - 1;

        out[i] <== a[i] * (1 - isEqResults[i+1]) + is_eqs[i].out * 1 + is_eq_lasts[i].out * 0x80;
    }
}

template BytesToBits(numBytes) {
    signal input bytes[numBytes];
    signal output bits[numBytes * 8];

    component decomps[numBytes];
    for(var i = 0; i < numBytes; i++) {
        decomps[i] = BitDecompose(8);
        decomps[i].num <== bytes[i];
        for(var j = 0; j < 8; j++) {
            bits[8 * i + j] <== decomps[i].bits[j];
        }
    }
}