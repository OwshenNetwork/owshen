pragma circom 2.1.5;

include "./utils/poseidon.circom";
include "./utils/utils.circom";

template Spend() {
    signal input balance;
    signal input salt;
    signal output coin;

    component coinHasher = Poseidon(2);
    coinHasher.inputs[0] <== balance;
    coinHasher.inputs[1] <== salt;
    coin <== coinHasher.out;

    signal input withdrawnBalance;
    signal input remainingCoinSalt;
    signal output remainingCoin;

    component sufficientBalanceChecker = GreaterEqThan(252);
    sufficientBalanceChecker.in[0] <== balance;
    sufficientBalanceChecker.in[1] <== withdrawnBalance;
    sufficientBalanceChecker.out === 1;


    component remainingCoinHasher = Poseidon(2);
    remainingCoinHasher.inputs[0] <== balance - withdrawnBalance;
    remainingCoinHasher.inputs[1] <== remainingCoinSalt;
    remainingCoin <== remainingCoinHasher.out;
}

component main {public [withdrawnBalance]} = Spend();