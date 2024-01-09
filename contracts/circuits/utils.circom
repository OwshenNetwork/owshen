pragma circom 2.0.0;

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

template LessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;

    component n2b = BitDecompose(n+1);

    n2b.num <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.bits[n];
}

template GreaterEqThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[1];
    lt.in[1] <== in[0]+1;
    lt.out ==> out;
}