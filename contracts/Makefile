.PHONY: test build

ENTROPY = 1234

build: circuits/reward_withdraw_cpp/ circuits/coin_withdraw_cpp/ circuits/coin_withdraw_0001.zkey circuits/reward_withdraw_0001.zkey circuits/coin_withdraw_verif_key.json circuits/reward_withdraw_verif_key.json src/CoinWithdrawVerifier.sol src/RewardWithdrawVerifier.sol
	forge build

test: build
	# Pass inputs
	echo '{"a": 11, "b": 3}' > circuits/coin_withdraw_input.json
	echo '{"a": 11, "b": 3}' > circuits/reward_withdraw_input.json

	cd circuits/coin_withdraw_cpp && make && ./coin_withdraw ../coin_withdraw_input.json ../coin_withdraw_witness.wtns
	cd circuits/reward_withdraw_cpp && make && ./reward_withdraw ../reward_withdraw_input.json ../reward_withdraw_witness.wtns

	# Generate proofs
	cd circuits && snarkjs groth16 prove coin_withdraw_0001.zkey coin_withdraw_witness.wtns coin_withdraw_proof.json coin_withdraw_public.json
	cd circuits && snarkjs groth16 prove reward_withdraw_0001.zkey reward_withdraw_witness.wtns reward_withdraw_proof.json reward_withdraw_public.json

	# Verify a proof on local machine
	cd circuits && snarkjs groth16 verify coin_withdraw_verif_key.json coin_withdraw_public.json coin_withdraw_proof.json
	cd circuits && snarkjs groth16 verify reward_withdraw_verif_key.json reward_withdraw_public.json reward_withdraw_proof.json

	cd circuits && snarkjs generatecall coin_withdraw_public.json coin_withdraw_proof.json | sed "s/\"0x/uint256\(0x/g" | sed "s/\"/\)/g" > CoinWithdraw_calldata
	cd circuits && snarkjs generatecall reward_withdraw_public.json reward_withdraw_proof.json | sed "s/\"0x/uint256\(0x/g" | sed "s/\"/\)/g" > RewardWithdraw_calldata

	for circ in CoinWithdraw RewardWithdraw; do \
		CALLDATA=$$(cat circuits/$${circ}_calldata); \
		echo "// SPDX-License-Identifier: UNLICENSED \n\
			pragma solidity ^0.8.13; \n\
			import \"forge-std/Test.sol\"; \n\
			import \"../src/$${circ}Verifier.sol\"; \n\
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
			}" | sed "s/PLACEHOLDER/$${CALLDATA}/g" > test/$${circ}Verifier.t.sol; \
	done;

	forge fmt
	forge test

circuits/coin_withdraw_verif_key.json: circuits/coin_withdraw_0001.zkey
	cd circuits && snarkjs zkey export verificationkey coin_withdraw_0001.zkey coin_withdraw_verif_key.json

circuits/reward_withdraw_verif_key.json: circuits/reward_withdraw_0001.zkey
	cd circuits && snarkjs zkey export verificationkey reward_withdraw_0001.zkey reward_withdraw_verif_key.json

circuits/coin_withdraw.r1cs circuits/coin_withdraw_cpp/: circuits/coin_withdraw.circom
	cd circuits && circom coin_withdraw.circom --r1cs --wasm --sym --c
	cd circuits/coin_withdraw_cpp && make

circuits/reward_withdraw.r1cs circuits/reward_withdraw_cpp/: circuits/reward_withdraw.circom
	cd circuits && circom reward_withdraw.circom --r1cs --wasm --sym --c
	cd circuits/reward_withdraw_cpp && make

circuits/coin_withdraw_0001.zkey: circuits/coin_withdraw.r1cs circuits/pot12_final.ptau
	cd circuits && snarkjs groth16 setup coin_withdraw.r1cs pot12_final.ptau coin_withdraw_0000.zkey
	cd circuits && snarkjs zkey contribute coin_withdraw_0000.zkey coin_withdraw_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

circuits/reward_withdraw_0001.zkey: circuits/reward_withdraw.r1cs circuits/pot12_final.ptau
	cd circuits && snarkjs groth16 setup reward_withdraw.r1cs pot12_final.ptau reward_withdraw_0000.zkey
	cd circuits && snarkjs zkey contribute reward_withdraw_0000.zkey reward_withdraw_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

src/CoinWithdrawVerifier.sol: circuits/coin_withdraw_0001.zkey
	cd circuits && snarkjs zkey export solidityverifier coin_withdraw_0001.zkey ../src/CoinWithdrawVerifier.sol

src/RewardWithdrawVerifier.sol: circuits/reward_withdraw_0001.zkey
	cd circuits && snarkjs zkey export solidityverifier reward_withdraw_0001.zkey ../src/RewardWithdrawVerifier.sol

circuits/pot12_final.ptau:
	# Generate Powers of Tau params
	cd circuits && snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
	cd circuits && snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" --entropy=${ENTROPY} -v
	cd circuits && snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

clean:
	rm -rf src/CoinWithdrawVerifier.sol test/CoinWithdrawVerifier.t.sol
	rm -rf src/RewardWithdrawVerifier.sol test/RewardWithdrawVerifier.t.sol
	cd circuits && rm -rf *.sym *.r1cs *.json *.zkey *.ptau *.wtns coin_withdraw_js/ coin_withdraw_cpp/ coin_withdraw_calldata reward_withdraw_calldata