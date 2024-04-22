.PHONY: test build

ENTROPY = 1234

build:  circuits/coin_withdraw_cpp/ circuits/coin_withdraw_0001.zkey circuits/coin_withdraw_verif_key.json src/CoinWithdrawVerifier.sol 
	forge build

test: build
	# Pass inputs
	echo '{"a": 11, "b": 3}' > circuits/coin_withdraw_input.json

	cd circuits/coin_withdraw_cpp && make && ./coin_withdraw ../coin_withdraw_input.json ../coin_withdraw_witness.wtns

	# Generate proofs
	cd circuits && snarkjs groth16 prove coin_withdraw_0001.zkey coin_withdraw_witness.wtns coin_withdraw_proof.json coin_withdraw_public.json

	# Verify a proof on local machine
	cd circuits && snarkjs groth16 verify coin_withdraw_verif_key.json coin_withdraw_public.json coin_withdraw_proof.json

	cd circuits && snarkjs generatecall coin_withdraw_public.json coin_withdraw_proof.json | sed "s/\"0x/uint256\(0x/g" | sed "s/\"/\)/g" > CoinWithdraw_calldata

	for circ in CoinWithdraw; do \
		CALLDATA=$$(cat circuits/$${circ}_calldata); \
		echo "// SPDX-License-Identifier: UNLICENSED \n\
			pragma solidity ^0.8.13; \n\
			import \"forge-std/Test.sol\"; \n\
			import \"../src/$${circ}Verifier.sol\"; \n\
			contract VerifierTest is Test { \n\
				$${circ}Verifier public verifier; \n\
				function setUp() public { \n\
					verifier = new $${circ}Verifier(); \n\
				} \n\
				function testVerifier() public view { \n\
				assert(verifier.verifyProof( \n\
						PLACEHOLDER \n\
					)); \n\
				} \n\
			}" | sed "s/PLACEHOLDER/$${CALLDATA}/g" > test/$${circ}Verifier.t.sol; \
	done;

	forge fmt
	forge test

circuits/coin_withdraw_verif_key.json: circuits/coin_withdraw_0001.zkey
	cd circuits && snarkjs zkey export verificationkey coin_withdraw_0001.zkey coin_withdraw_verif_key.json


circuits/coin_withdraw.r1cs circuits/coin_withdraw_cpp/: circuits/coin_withdraw.circom
	cd circuits && circom coin_withdraw.circom --r1cs --wasm --sym --c
	cd circuits/coin_withdraw_cpp && make

circuits/coin_withdraw_0001.zkey: circuits/coin_withdraw.r1cs
	cd circuits && snarkjs groth16 setup coin_withdraw.r1cs pot_final.ptau coin_withdraw_0000.zkey
	cd circuits && snarkjs zkey contribute coin_withdraw_0000.zkey coin_withdraw_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

src/CoinWithdrawVerifier.sol: circuits/coin_withdraw_0001.zkey
	cd circuits && snarkjs zkey export solidityverifier coin_withdraw_0001.zkey ../src/CoinWithdrawVerifier.sol
	sed -i 's/Groth16Verifier/CoinWithdrawVerifier/' src/CoinWithdrawVerifier.sol

clean:
	rm -rf src/CoinWithdrawVerifier.sol test/CoinWithdrawVerifier.t.sol src/MptLastVerifier.sol test/MptLastVerifier.t.sol
	cd circuits && rm -rf *.sym *.r1cs *.json *.zkey *.wtns coin_withdraw_js/ coin_withdraw_cpp/ coin_withdraw_calldata