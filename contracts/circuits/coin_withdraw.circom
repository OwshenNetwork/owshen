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

template CoinWithdraw() {
    signal input index;
    signal input token_address;
    signal input amount;
    signal input new_amount1;
    signal input new_amount2;
    signal input pk_ax1;
    signal input pk_ay1;
    signal input pk_ax2;
    signal input pk_ay2;
    signal input secret;
    signal input proof[16][3];
    signal output root;
    signal output nullifier;
    signal output new_commitment1;  
    signal output new_commitment2;

    signal inters[17];

    component bd = BitDecompose(32);
    bd.num <== index;
    
    component pk = BabyPbk();
    pk.in <== secret;

    component gt1 = GreaterEqThan(250);

    gt1.in[0] <== amount;
    gt1.in[1] <== new_amount1;
    gt1.out === 1;
    
    component gt2 = GreaterEqThan(250);

    gt2.in[0] <== amount;
    gt2.in[1] <== new_amount2;
    gt2.out === 1;

    amount === new_amount1 + new_amount2;

    component commiter_hasher = Poseidon(4);
    commiter_hasher.inputs[0] <== pk.Ax;
    commiter_hasher.inputs[1] <== pk.Ay;
    commiter_hasher.inputs[2] <== amount;
    commiter_hasher.inputs[3] <== token_address;
    inters[0] <== commiter_hasher.out;

    component new_commiter_hasher_1 = Poseidon(4);
    new_commiter_hasher_1.inputs[0] <== pk_ax1;
    new_commiter_hasher_1.inputs[1] <== pk_ay1;
    new_commiter_hasher_1.inputs[2] <== new_amount1;
    new_commiter_hasher_1.inputs[3] <== token_address;
    new_commitment1 <== new_commiter_hasher_1.out;

    component new_commiter_hasher_2 = Poseidon(4);
    new_commiter_hasher_2.inputs[0] <== pk_ax2;
    new_commiter_hasher_2.inputs[1] <== pk_ay2;
    new_commiter_hasher_2.inputs[2] <== new_amount2;
    new_commiter_hasher_2.inputs[3] <== token_address;
    new_commitment2 <== new_commiter_hasher_2.out;
    
    component nullifier_hasher = Poseidon(4); // TODO: Switch to Poseidon2
    nullifier_hasher.inputs[0] <== secret;
    nullifier_hasher.inputs[1] <== index;
    nullifier_hasher.inputs[2] <== 0;
    nullifier_hasher.inputs[3] <== 0;
    nullifier <== nullifier_hasher.out;

    component hashers[16];
    component swaps[16];
    for(var i=0; i < 16; i++) {
        swaps[i] = CSwap();
        swaps[i].s[0] <== bd.bits[2 * i];
        swaps[i].s[1] <== bd.bits[2 * i + 1];
        swaps[i].v <== inters[i];
        swaps[i].in <== proof[i];

        hashers[i] = Poseidon(4);
        hashers[i].inputs <== swaps[i].out;
        inters[i+1] <== hashers[i].out;
    }
    
    root <== inters[16];
 }

 component main = CoinWithdraw();