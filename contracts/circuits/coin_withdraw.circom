pragma circom 2.0.0;

include "hasher.circom";

template CSwap() {
    signal input a;
    signal input b;
    signal input swap;
    signal output l;
    signal output r;
    l <== (b - a) * swap + a;
    r <== (a - b) * swap + b;
}

template BitDecompose(N) {
    signal input num;
    signal output bits[N];
    var pow = 1;
    var i = 0;
    var total = 0;
    for(i=0; i<N; i++) {
        bits[i] <-- (num >> i) & 1;
        bits[i] * (bits[i] - 1) === 0;
        total += pow * bits[i];
        pow = pow * 2;
    }
    total === num;
}

template CoinWithdraw() {
    signal input index;
    signal input value;
    signal input proof[32];
    signal output root;

    component bd = BitDecompose(32);
    component swaps[32];
    bd.num <== index;
    component hashers[32];
    signal inters[33];

    inters[0] <== value;

    var i = 0;

    for(i=0; i < 32; i++) {
        swaps[i] = CSwap();
        swaps[i].swap <== bd.bits[i];
        swaps[i].a <== inters[i];
        swaps[i].b <== proof[i];

        hashers[i] = Hasher();
        hashers[i].left <== swaps[i].l;
        hashers[i].right <== swaps[i].r;
        inters[i+1] <== hashers[i].hash;
    }
    
    root <== inters[32];
 }

 component main = CoinWithdraw();