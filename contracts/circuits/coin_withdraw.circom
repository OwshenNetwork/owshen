pragma circom 2.0.0;

include "poseidon.circom";
include "utils.circom";
include "babyjub.circom";


template CoinWithdraw(NUM_INPS, NUM_OUTS, BATCH_SIZE) {
    signal input token_address;

    signal input index[NUM_INPS];
    signal input amount[NUM_INPS];
    signal input secret[NUM_INPS];

    signal input user_checkpoint_head;
    signal input user_latest_values_commitment_head;
    signal input value[NUM_INPS];
    signal input between_values[NUM_INPS][BATCH_SIZE];
    signal input checkpoint_commitments[BATCH_SIZE];
    signal input checkpoints[BATCH_SIZE];
    signal input latest_values[BATCH_SIZE];
    signal input is_in_latest_commits[NUM_INPS];

    signal input new_amount[NUM_OUTS];
    signal input pk_ax[NUM_OUTS];
    signal input pk_ay[NUM_OUTS];

    signal output checkpoint_head;
    signal output latest_values_commitment_head;
    signal output nullifier[NUM_INPS];
    signal output new_commitment[NUM_OUTS];

    signal total_amount;

    var sum_amount = 0;
    signal masked_amounts[NUM_INPS];
    component is_zero[NUM_INPS];
    for(var i = 0; i < NUM_INPS; i++) {
        is_zero[i] = IsZero();
        is_zero[i].in <== secret[i];

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

    component pk[NUM_INPS];
    component commiter_hasher[NUM_INPS];
    component nullifier_hasher[NUM_INPS];
    signal between_commitments[NUM_INPS][BATCH_SIZE];
    component between_commitments_hash[NUM_INPS][BATCH_SIZE];
    component is_between_commitment_contained[NUM_INPS];
    component is_value_contained[NUM_INPS];

    component checkpoints_hasher[BATCH_SIZE];
    signal checkpoints_hash[BATCH_SIZE];
    checkpoints_hash[0] <== checkpoints[0];
    for(var j = 1; j < BATCH_SIZE; j++) {
        checkpoints_hasher[j] = Poseidon(2);
        checkpoints_hasher[j].inputs[0] <== checkpoints_hash[j - 1];
        checkpoints_hasher[j].inputs[1] <== checkpoint_commitments[j];
        checkpoints_hash[j] <== checkpoints_hasher[j].out;
    }
    component is_head_contained = Contains(BATCH_SIZE);
    is_head_contained.value <== user_checkpoint_head;
    is_head_contained.values <== checkpoints_hash;
    is_head_contained.out === 1;
    checkpoint_head <== user_checkpoint_head;

    component latest_values_hasher[BATCH_SIZE];
    signal latest_values_hash[BATCH_SIZE];
    component is_user_values_contained_in_latest_values[NUM_INPS];
    latest_values_hash[0] <== latest_values[0];
    for(var i = 1; i < BATCH_SIZE; i++) {
        latest_values_hasher[i] = Poseidon(2);
        latest_values_hasher[i].inputs[0] <== latest_values_hash[i - 1];
        latest_values_hasher[i].inputs[1] <== latest_values[i];
        latest_values_hash[i] <== latest_values_hasher[i].out;
    }
    component is_latest_values_head_contained = Contains(BATCH_SIZE);
    is_latest_values_head_contained.value <== user_latest_values_commitment_head;
    is_latest_values_head_contained.values <== latest_values_hash;
    is_latest_values_head_contained.out === 1;
    latest_values_commitment_head <== user_latest_values_commitment_head;

    for(var i = 0; i < NUM_INPS; i++) {
        pk[i] = BabyPbk();
        pk[i].in <== secret[i];

        commiter_hasher[i] = Poseidon(4);
        commiter_hasher[i].inputs[0] <== pk[i].Ax;
        commiter_hasher[i].inputs[1] <== pk[i].Ay;
        commiter_hasher[i].inputs[2] <== amount[i];
        commiter_hasher[i].inputs[3] <== token_address;
        // TODO: validate commitment not just calculation
        // value[i] === (1 - is_zero[i].out) * commiter_hasher[i].out;

        nullifier_hasher[i] = Poseidon(4); // TODO: Switch to Poseidon2
        nullifier_hasher[i].inputs[0] <== secret[i];
        nullifier_hasher[i].inputs[1] <== index[i];
        nullifier_hasher[i].inputs[2] <== 0;
        nullifier_hasher[i].inputs[3] <== 0;
        nullifier[i] <== (1 - is_zero[i].out) * nullifier_hasher[i].out;

        between_commitments[i][0] <== between_values[i][0];
        for(var j = 1; j < BATCH_SIZE; j++) {
            between_commitments_hash[i][j] = Poseidon(2);
            between_commitments_hash[i][j].inputs[0] <== between_commitments[i][j - 1];
            between_commitments_hash[i][j].inputs[1] <== between_values[i][j];
            between_commitments[i][j] <== between_commitments_hash[i][j].out;
        }

        is_between_commitment_contained[i] = Contains(BATCH_SIZE);
        is_between_commitment_contained[i].value <== between_commitments[i][BATCH_SIZE - 1];
        is_between_commitment_contained[i].values <== checkpoint_commitments;
        is_between_commitment_contained[i].out === 1 * (1 - is_in_latest_commits[i]);

        is_user_values_contained_in_latest_values[i] = Contains(BATCH_SIZE);
        is_user_values_contained_in_latest_values[i].value <== value[i];
        is_user_values_contained_in_latest_values[i].values <== latest_values;
        is_user_values_contained_in_latest_values[i].out === 1 * is_in_latest_commits[i];
    }
 }

 component main = CoinWithdraw(2, 2, 1024);
 