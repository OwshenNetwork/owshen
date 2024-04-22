pragma circom 2.1.5;

include "./hasher.circom";

template Bytes2Num(len, startIndex, nBytes) {
    signal input block[len];
    signal output out;

    var result = 0;
    for(var i = 0; i < nBytes; i++) {
        result += (256**i) * block[startIndex + i];
    }

    out <== result;
}

template BytesToNums(numBytes, bytesPerNum) {
    var cnt = numBytes \ bytesPerNum + (numBytes % bytesPerNum != 0 ? 1 : 0);

    signal input inp[numBytes];
    signal output out[cnt];

    component converters[cnt];
    for(var i = 0; i < cnt; i++) {
        converters[i] = Bytes2Num(numBytes, i * bytesPerNum, numBytes - i * bytesPerNum < bytesPerNum ? numBytes - i * bytesPerNum : bytesPerNum);
        converters[i].block <== inp;
        out[i] <== converters[i].out;
    }
}

template HashBytes(numBytes, bytesPerNum) {
    signal input inp[numBytes];
    signal output out;

    var cnt = numBytes \ bytesPerNum + (numBytes % bytesPerNum != 0 ? 1 : 0);

    component tonums = BytesToNums(numBytes, bytesPerNum);
    tonums.inp <== inp;

    component hashers[cnt];
    signal commits[cnt+1];
    commits[0] <== 0;
    for(var i = 0; i < cnt; i++) {
        hashers[i] = Hasher();
        hashers[i].left <== commits[i];
        hashers[i].right <== tonums.out[i];
        commits[i+1] <== hashers[i].hash;
    }

    out <== commits[cnt];
}