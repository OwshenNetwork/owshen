// Keccak256 hash function (ethereum version).
// For LICENSE check https://github.com/vocdoni/keccak256-circom/blob/master/LICENSE

pragma circom 2.1.5;

include "./utils.circom";
include "../selector.circom";
include "./permutations.circom";

template KeccakfRound(r) {
    signal input in[25*64];
    signal output out[25*64];
    var i;

    component theta = Theta();
    component rhopi = RhoPi();
    component chi = Chi();
    component iota = Iota(r);

    for (i=0; i<25*64; i++) {
        theta.in[i] <== in[i];
    }
    for (i=0; i<25*64; i++) {
        rhopi.in[i] <== theta.out[i];
    }
    for (i=0; i<25*64; i++) {
        chi.in[i] <== rhopi.out[i];
    }
    for (i=0; i<25*64; i++) {
        iota.in[i] <== chi.out[i];
    }
    for (i=0; i<25*64; i++) {
        out[i] <== iota.out[i];
    }
}

template Absorb() {
    var blockSizeBytes=136;

    signal input s[25*64];
    signal input block[blockSizeBytes*8];
    signal output out[25*64];
    var i;
    var j;

    component aux[blockSizeBytes/8];
    component newS = Keccakf();

    for (i=0; i<blockSizeBytes/8; i++) {
        aux[i] = XorArray(64);
        for (j=0; j<64; j++) {
            aux[i].a[j] <== s[i*64+j];
            aux[i].b[j] <== block[i*64+j];
        }
        for (j=0; j<64; j++) {
            newS.in[i*64+j] <== aux[i].out[j];
        }
    }
    // fill the missing s that was not covered by the loop over
    // blockSizeBytes/8
    for (i=(blockSizeBytes/8)*64; i<25*64; i++) {
            newS.in[i] <== s[i];
    }
    for (i=0; i<25*64; i++) {
        out[i] <== newS.out[i];
    }
}

template Final(nBlocksIn) {
    signal input in[nBlocksIn * 136 * 8];
    signal input blocks;
    signal output out[25*64];
    var blockSize=136*8;
    var i;
    var b;

    component abs[nBlocksIn];

    for (b=0; b<nBlocksIn; b++) {
        abs[b] = Absorb();
        if (b == 0) {
            for (i=0; i<25*64; i++) {
                abs[b].s[i] <== 0;
            }
        } else {
            for (i=0; i<25*64; i++) {
                abs[b].s[i] <== abs[b-1].out[i];
            }
        }
        for (i=0; i<blockSize; i++) {
            abs[b].block[i] <== in[b * 136 * 8 + i];
        }
    }

    component selectors[25*64];

    for (i=0; i<25*64; i++) {
        selectors[i] = Selector(nBlocksIn);
        selectors[i].select <== blocks - 1;
        for(var j = 0; j < nBlocksIn; j++) {
            selectors[i].vals[j] <== abs[j].out[i];
        }
        out[i] <== selectors[i].out;
    }
}

template Squeeze(nBits) {
    signal input s[25*64];
    signal output out[nBits];
    var i;
    var j;

    for (i=0; i<25; i++) {
        for (j=0; j<64; j++) {
            if (i*64+j<nBits) {
                out[i*64+j] <== s[i*64+j];
            }
        }
    }
}

template Keccakf() {
    signal input in[25*64];
    signal output out[25*64];
    var i;
    var j;

    // 24 rounds
    component round[24];
    signal midRound[24*25*64];
    for (i=0; i<24; i++) {
        round[i] = KeccakfRound(i);
        if (i==0) {
            for (j=0; j<25*64; j++) {
                midRound[j] <== in[j];
            }
        }
        for (j=0; j<25*64; j++) {
            round[i].in[j] <== midRound[i*25*64+j];
        }
        if (i<23) {
            for (j=0; j<25*64; j++) {
                midRound[(i+1)*25*64+j] <== round[i].out[j];
            }
        }
    }

    for (i=0; i<25*64; i++) {
        out[i] <== round[23].out[i];
    }
}

template Keccak(nBlocksIn) {
    signal input in[nBlocksIn * 136 * 8];
    signal input blocks;
    signal output out[32 * 8];
    var i;

    component f = Final(nBlocksIn);
    f.blocks <== blocks;
    for (i=0; i<nBlocksIn * 136 * 8; i++) {
        f.in[i] <== in[i];
    }
    component squeeze = Squeeze(32 * 8);
    for (i=0; i<25*64; i++) {
        squeeze.s[i] <== f.out[i];
    }
    for (i=0; i<32 * 8; i++) {
        out[i] <== squeeze.out[i];
    }
}