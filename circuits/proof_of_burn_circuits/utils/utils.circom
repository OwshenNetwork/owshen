
pragma circom 2.1.5;

template IsZero() {
    signal input in;
    signal output out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv +1;
    in*out === 0;
}

template LessEqThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[0];
    lt.in[1] <== in[1]+1;
    lt.out ==> out;
}

template LessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;

    component n2b = BitDecompose(n+1);

    n2b.num <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.bits[n];
}

// N is the number of bits the input  have.
// The MSF is the sign bit.
template GreaterEqThan(n) {
    signal input in[2];
    signal output out;

    component lt = LessThan(n);

    lt.in[0] <== in[1];
    lt.in[1] <== in[0]+1;
    lt.out ==> out;
}

template IsEqual() {
    signal input in[2];
    signal output out;

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    isz.out ==> out;
}

template RangeCheck(n) {
    signal input inp;
    signal output out;

    signal select_conds[n+1];
    select_conds[0] <== 1;
    for(var i = 0; i < n; i++) {
        select_conds[i+1] <== select_conds[i] * (inp - i);
    }
    
    component isz = IsZero();
    isz.in <== select_conds[n];
    out <== isz.out;
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

template ByteDecompose(N) { 
    signal input num;
    signal output bytes[N];
    var pow = 1;
    var total = 0;
    component bd[N];
    for (var i = 0; i < N; i++) {
        bytes[i] <-- (num >> (8 * i)) & 0xFF;
        bd[i] = BitDecompose(8);
        bd[i].num <==  bytes[i];
        total += pow * bytes[i];
        pow = pow * 256; 
    }

    total === num; 
}

template GetRealByteLength(N) {
    signal input bytes[N];
    signal output len;

    component isZero[N];

    signal isZeroResult[N+1];
    isZeroResult[0] <== 1;

    for (var i = 0; i < N; i++) {
        isZero[i] = IsZero();
        isZero[i].in <== bytes[N-i-1];
        isZeroResult[i+1] <== isZero[i].out * isZeroResult[i];
    }
    
    var total = 0;
    
    for (var j = 1; j < N + 1; j++) {
        total = total + isZeroResult[j];
    }

    len <== N - total;
}

template IfThenElse() {
    signal input condition; 
    signal input ifTrue;
    signal input ifFalse;
    signal output out;


    signal intermediateTrue;
    signal intermediateFalse;

    intermediateTrue <== condition * ifTrue;
    intermediateFalse <== (1 - condition) * ifFalse;

    out <== intermediateTrue + intermediateFalse;
}


template CompConstant(ct) {
    signal input in[254];
    signal output out;

    signal parts[127];
    signal sout;

    var clsb;
    var cmsb;
    var slsb;
    var smsb;

    var sum=0;

    var b = (1 << 128) -1;
    var a = 1;
    var e = 1;
    var i;

    for (i=0;i<127; i++) {
        clsb = (ct >> (i*2)) & 1;
        cmsb = (ct >> (i*2+1)) & 1;
        slsb = in[i*2];
        smsb = in[i*2+1];

        if ((cmsb==0)&&(clsb==0)) {
            parts[i] <== -b*smsb*slsb + b*smsb + b*slsb;
        } else if ((cmsb==0)&&(clsb==1)) {
            parts[i] <== a*smsb*slsb - a*slsb + b*smsb - a*smsb + a;
        } else if ((cmsb==1)&&(clsb==0)) {
            parts[i] <== b*smsb*slsb - a*smsb + a;
        } else {
            parts[i] <== -a*smsb*slsb + a;
        }

        sum = sum + parts[i];

        b = b -e;
        a = a +e;
        e = e*2;
    }

    sout <== sum;

    component num2bits = Num2Bits(135);

    num2bits.in <== sout;

    out <== num2bits.out[127];
}

template AliasCheck() {

    signal input in[254];

    component  compConstant = CompConstant(-1);

    for (var i=0; i<254; i++) in[i] ==> compConstant.in[i];

    compConstant.out === 0;
}

template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1=0;

    var e2=1;
    for (var i = 0; i<n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] -1 ) === 0;
        lc1 += out[i] * e2;
        e2 = e2+e2;
    }

    lc1 === in;
}   

template Bits2Num(n) {
    signal input in[n];
    signal output out;
    var lc1=0;

    var e2 = 1;
    for (var i = 0; i<n; i++) {
        lc1 += in[i] * e2;
        e2 = e2 + e2;
    }

    lc1 ==> out;
}

template Num2Bits_strict() {
    signal input in;
    signal output out[254];

    component aliasCheck = AliasCheck();
    component n2b = Num2Bits(254);
    in ==> n2b.in;

    for (var i=0; i<254; i++) {
        n2b.out[i] ==> out[i];
        n2b.out[i] ==> aliasCheck.in[i];
    }
}

template Bits2NumBigendian(n) {
    signal input in[n];
    signal output out;
    var number = 0;
    var step1 = 1;

    for (var i = n / 8 - 1; i >= 0; i--) {
        var step2 = 1;
        var tmp_number = 0;
        for (var j = 0; j < 8; j++) {
            tmp_number += in[i * 8 + j] * step2;
            step2 *= 2;
        }
        number += tmp_number * step1;
        step1 *= 256;
    }

    number ==> out;
}
