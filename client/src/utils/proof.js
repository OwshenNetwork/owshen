//"Usage: node generate_witness.js <file.wasm> <input.json> <output.wtns>"
import { builder } from "./witness_calc";

// get proper witness.bin
export const generate_witness = async (
  witnessCalculator,
  wasm,
  input,
  wasm_file
) => {
  const parsedInput = JSON.parse(input);
  const response = await fetch(wasm_file);
  const buffer = await response.arrayBuffer();

  let result = builder(buffer).then(async (witnessCalculator) => {
    return await witnessCalculator.calculateWTNSBin(parsedInput, 0);
  });

  return result;
};
