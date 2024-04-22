pragma circom 2.1.5;

include "./keccak/keccak.circom";
include "./keccak/utils.circom";
include "./utils.circom";


 template substringCheck (nBlocks, blockSize, subLen) {
    signal input mainInput[nBlocks * blockSize];
    signal input numBlocks;
    signal input subInput[subLen];
    signal output out;

    component range_check = RangeCheck(nBlocks);
    range_check.inp <== numBlocks - 1;
    range_check.out === 1;

    // A = 2^0 subInput[0] + 2^1 subInput[1] + ... + 2^255 subInput[255]

    signal A[subLen + 1];
    A[0] <== 0;
    for (var i = 0; i < subLen; i++) {
        A[i+1] <== subInput[i] * (2**i) + A[i];
    }

    signal B[nBlocks * blockSize + 1];
    B[0] <== 0;

    for (var i = 0; i < nBlocks * blockSize; i++) {
        B[i+1] <== mainInput[i] * (2**i) + B[i];
    }
    
    component eq[nBlocks * blockSize - subLen + 1];
    component range_checkers[nBlocks * blockSize - subLen + 1];

    signal eq_check[nBlocks * blockSize - subLen + 2];

    eq_check[0] <== 0;
    for (var i = 0; i < nBlocks * blockSize - subLen + 1; i++) {
        eq[i] = IsEqual();
        eq[i].in[0] <== A[subLen] * (2 ** i);
        eq[i].in[1] <== B[i+subLen] - B[i];

        // Check if (numBlocks - 1 - block) overflows (Check if block < numBlocks)
        var block = (i + subLen - 1) \ blockSize;
        range_checkers[i] = RangeCheck(nBlocks);
        range_checkers[i].inp <== numBlocks - 1 - block;

        eq_check[i+1] <== eq_check[i] + eq[i].out * range_checkers[i].out;
    }

    component isz = IsZero();
    isz.in <== eq_check[nBlocks * blockSize - subLen + 1];
    out <== 1 - isz.out;
 }