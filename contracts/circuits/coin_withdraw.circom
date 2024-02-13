pragma circom 2.0.0;

include "poseidon.circom";
include "utils.circom";
include "babyjub.circom";

template CSwap() {
    signal input in[3];
    signal input s[2];
    signal input v;
    signal output out[4];

    signal s0_and_s1;
    signal s0_or_s1;
    s0_and_s1 <== s[0] * s[1];
    s0_or_s1 <== s[0] + s[1] - s0_and_s1;

    signal out1p;
    signal out2p;

    out[0] <== (in[0] - v) * s0_or_s1 + v;
    out1p <== (v - in[0]) * s[0] + in[0];
    out[1] <== (in[1] - out1p) * s[1] + out1p;
    out2p <== (in[2] - v) * s[0] + v;
    out[2] <== (out2p - in[1]) * s[1] + in[1];
    out[3] <== (v - in[2]) * s0_and_s1 + in[2];
}

template CoinWithdraw(NUM_INPS, NUM_OUTS) {
    signal input token_address;

    signal input index[NUM_INPS];
    signal input amount[NUM_INPS];
    signal input secret[NUM_INPS];
    signal input proof[NUM_INPS][16][3];

    signal input new_amount[NUM_OUTS];
    signal input pk_ax[NUM_OUTS];
    signal input pk_ay[NUM_OUTS];

    signal output root;
    signal output nullifier[NUM_INPS];
    signal output new_commitment[NUM_OUTS];

    signal total_amount;

    var sum_amount = 0;
    signal masked_amounts[NUM_INPS];
    component is_zero[NUM_INPS];
    for(var i = 0; i < NUM_INPS; i++) {
        is_zero[i] = IsZero();
        is_zero[i].in <== index[i];

        masked_amounts[i] <== amount[i] * (1 - is_zero[i].out);
        sum_amount += masked_amounts[i];
    }
    total_amount <== sum_amount;

    var sum_new_amount = 0;
    component gt[NUM_OUTS];
    component new_commiter_hasher[NUM_OUTS];
    for(var i = 0; i < NUM_OUTS; i++) {
        gt[i] = GreaterEqThan(250);
        gt[i].in[0] <== total_amount;
        gt[i].in[1] <== new_amount[i];
        gt[i].out === 1;

        new_commiter_hasher[i] = Poseidon(4);
        new_commiter_hasher[i].inputs[0] <== pk_ax[i];
        new_commiter_hasher[i].inputs[1] <== pk_ay[i];
        new_commiter_hasher[i].inputs[2] <== new_amount[i];
        new_commiter_hasher[i].inputs[3] <== token_address;
        new_commitment[i] <== new_commiter_hasher[i].out;

        sum_new_amount += new_amount[i];
    }
    total_amount === sum_new_amount;

    signal inters[NUM_INPS][17];

    component bd[NUM_INPS];
    component pk[NUM_INPS];
    component hashers[NUM_INPS][16];
    component swaps[NUM_INPS][16];
    component commiter_hasher[NUM_INPS];
    component nullifier_hasher[NUM_INPS];
    for(var i = 0; i < NUM_INPS; i++) {
        bd[i] = BitDecompose(32);
        bd[i].num <== index[i];

        pk[i] = BabyPbk();
        pk[i].in <== secret[i];

        commiter_hasher[i] = Poseidon(4);
        commiter_hasher[i].inputs[0] <== pk[i].Ax;
        commiter_hasher[i].inputs[1] <== pk[i].Ay;
        commiter_hasher[i].inputs[2] <== amount[i];
        commiter_hasher[i].inputs[3] <== token_address;
        inters[i][0] <== commiter_hasher[i].out;

        nullifier_hasher[i] = Poseidon(4); // TODO: Switch to Poseidon2
        nullifier_hasher[i].inputs[0] <== secret[i];
        nullifier_hasher[i].inputs[1] <== index[i];
        nullifier_hasher[i].inputs[2] <== 0;
        nullifier_hasher[i].inputs[3] <== 0;
        nullifier[i] <== (1 - is_zero[i].out) * nullifier_hasher[i].out;

        for(var j = 0; j < 16; j++) {
            swaps[i][j] = CSwap();
            swaps[i][j].s[0] <== bd[i].bits[2 * j];
            swaps[i][j].s[1] <== bd[i].bits[2 * j + 1];
            swaps[i][j].v <== inters[i][j];
            swaps[i][j].in <== proof[i][j];

            hashers[i][j] = Poseidon(4);
            hashers[i][j].inputs <== swaps[i][j].out;
            inters[i][j+1] <== hashers[i][j].out;
        }

        if(i == 0) {
            root <== inters[i][16];
        }

        0 === (root - inters[i][16]) * (1 - is_zero[i].out);
    }
 }

 component main = CoinWithdraw(2, 2);
 