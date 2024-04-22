pragma circom 2.1.5;


// Returns vals[select]
template Selector (n) {
    signal input vals[n];
    signal input select;
    signal output out;

    // Limit select (0<=select<n)
    component range_check = RangeCheck(n);
    range_check.inp <== select;
    range_check.out === 1;

    component eq_checkers[n];
    signal eq_val_sums[n+1];
    eq_val_sums[0] <== 0;
    for(var i = 0; i < n; i++) {
        eq_checkers[i] = IsEqual();
        eq_checkers[i].in[0] <== select;
        eq_checkers[i].in[1] <== i;
        eq_val_sums[i+1] <== eq_val_sums[i] + eq_checkers[i].out * vals[i];
    }

    out <== eq_val_sums[n];
}
