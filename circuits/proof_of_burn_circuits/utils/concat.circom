pragma circom 2.1.5;

include "./utils.circom";

template Mask(n) {
    signal input in[n];
    signal input ind;
    signal output out[n];

    signal eqs[n+1];
    eqs[0] <== 1;
    component eqcomps[n];
    for(var i = 0; i < n; i++) {
        eqcomps[i] = IsEqual();
        eqcomps[i].in[0] <== i;
        eqcomps[i].in[1] <== ind;
        eqs[i+1] <== eqs[i] * (1 - eqcomps[i].out);
    }

    for(var i = 0; i < n; i++) {
        out[i] <== in[i] * eqs[i+1];
    }
}

template Shift(n, maxShift) {
    signal input in[n];
    signal input count;
    signal output out[n + maxShift];

    var outsum[n + maxShift];

    component eqcomps[maxShift + 1];
    signal temps[maxShift + 1][n];
    for(var i = 0; i <= maxShift; i++) {
        eqcomps[i] = IsEqual();
        eqcomps[i].in[0] <== i;
        eqcomps[i].in[1] <== count;
        for(var j = 0; j < n; j++) {
            temps[i][j] <== eqcomps[i].out * in[j];
            outsum[i + j] += temps[i][j];
        }
    }

    for(var i = 0; i < n + maxShift; i++) {
        out[i] <== outsum[i];
    }
}

template Concat(maxLenA, maxLenB) {
    signal input a[maxLenA];
    signal input aLen;

    signal input b[maxLenB];
    signal input bLen;

    signal output out[maxLenA + maxLenB];
    signal output outLen;

    component aLenChecker = LessEqThan(10);
    aLenChecker.in[0] <== aLen;
    aLenChecker.in[1] <== maxLenA;
    aLenChecker.out === 1;

    component bLenChecker = LessEqThan(10);
    bLenChecker.in[0] <== bLen;
    bLenChecker.in[1] <== maxLenB;
    bLenChecker.out === 1;

    component aMasker = Mask(maxLenA);
    aMasker.in <== a;
    aMasker.ind <== aLen;

    component bMasker = Mask(maxLenB);
    bMasker.in <== b;
    bMasker.ind <== bLen;

    var outVals[maxLenA + maxLenB];

    component bShifter = Shift(maxLenB, maxLenA);
    bShifter.count <== aLen;
    bShifter.in <== bMasker.out;

    for(var i = 0; i < maxLenA; i++) {
        outVals[i] += aMasker.out[i];
    }

    for(var i = 0; i < maxLenA + maxLenB; i++) {
        outVals[i] += bShifter.out[i];
    }

    for(var i = 0; i < maxLenA + maxLenB; i++) {
        out[i] <== outVals[i];
    }

    outLen <== aLen + bLen;
}

template ReverseArray(N) {
    signal input bytes[N];
    signal input realByteLen;
    signal output out[N];

    var lenDiff = N - realByteLen;
    signal reversed[N];

    component shifter = Shift(N, N);
    shifter.count <== lenDiff;
    shifter.in <== bytes; 

   for(var i = 0; i < N; i++) {
        reversed[i] <== shifter.out[N - i - 1];
    }

    out <== reversed;
}