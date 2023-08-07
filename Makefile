.PHONY: test

test:
	cd circuits && circom owshen.circom --r1cs --wasm --sym --c
	cd circuits/owshen_cpp && make

	# Pass inputs
	echo '{"a": 11, "b": 3}' > circuits/input.json

	cd circuits/owshen_cpp && ./owshen ../input.json ../witness.wtns

	# Generate Powers of Tau params
	cd circuits && snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
	cd circuits && snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v
	cd circuits && snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

	# Generate Phase2 params
	cd circuits && snarkjs groth16 setup owshen.r1cs pot12_final.ptau owshen_0000.zkey
	cd circuits && snarkjs zkey contribute owshen_0000.zkey owshen_0001.zkey --name="1st Contributor Name" -v

	# Generate verification key
	cd circuits && snarkjs zkey export verificationkey owshen_0001.zkey verification_key.json

	# Generate proof
	cd circuits && snarkjs groth16 prove owshen_0001.zkey witness.wtns proof.json public.json

	# Verify a proof on local machine
	cd circuits && snarkjs groth16 verify verification_key.json public.json proof.json

	# Generate Solidity contract
	cd circuits && snarkjs zkey export solidityverifier owshen_0001.zkey ../src/Verifier.sol

	cd circuits && snarkjs generatecall | sed "s/\"0x/uint256\(0x/g" | sed "s/\"/\)/g" > calldata

	# Generate test/Verifier.t.sol
	CALLDATA=$$(cat circuits/calldata); \
	echo "// SPDX-License-Identifier: UNLICENSED \n\
		pragma solidity ^0.8.13; \n\
		import \"forge-std/Test.sol\"; \n\
		import \"../src/Verifier.sol\"; \n\
		contract VerifierTest is Test { \n\
			Groth16Verifier public verifier; \n\
			function setUp() public { \n\
				verifier = new Groth16Verifier(); \n\
			} \n\
			function testVerifier() public view { \n\
			assert(verifier.verifyProof( \n\
					PLACEHOLDER \n\
				)); \n\
			} \n\
		}" | sed "s/PLACEHOLDER/$${CALLDATA}/g" > test/Verifier.t.sol
	
	forge fmt
	forge test

clean:
	rm -rf src/Verifier.sol test/Verifier.t.sol
	cd circuits && rm -rf *.sym *.r1cs *.json *.zkey *.ptau *.wtns calldata owshen_js/ owshen_cpp/