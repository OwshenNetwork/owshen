import snarkjs from "snarkjs";
import fs from "fs";

export const prove = async (
  tokenAddress,
  index,
  amount,
  secret,
  proof,
  newAmount,
  pk,
  paramsPath,
  witnessGenPath
) => {
  try {
    // Convert inputs to a format suitable for snarkjs
    const inputs = {
      tokenAddress,
      index,
      amount,
      secret,
      proof: proof.map((p) => p.map((q) => q.map((r) => r.toString(10)))),
      new_amount: newAmount.map((na) => na.toString(10)),
      pk_ax: pk.map((key) => key.point.x.toString(10)),
      pk_ay: pk.map((key) => key.point.y.toString(10)),
    };

    const { proof: zkProof, publicSignals } = await snarkjs.groth16.fullProve(
      inputs,
      paramsPath,
      witnessGenPath
    );

    console.log("Proof:", zkProof);
    console.log("Public signals:", publicSignals);

    return zkProof;
  } catch (error) {
    console.error("Error generating zk-SNARK proof:", error);
    throw error;
  }
};
