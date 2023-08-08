.PHONY: test

test: circuits/reward_withdraw_cpp/ circuits/coin_withdraw_cpp/ circuits/coin_withdraw_0001.zkey circuits/reward_withdraw_0001.zkey circuits/coin_withdraw_verif_key.json circuits/reward_withdraw_verif_key.json
	# Pass inputs
	echo '{"a": 11, "b": 3}' > circuits/coin_withdraw_input.json
	echo '{"a": 11, "b": 3}' > circuits/reward_withdraw_input.json

	cd circuits/coin_withdraw_cpp && ./coin_withdraw ../coin_withdraw_input.json ../coin_withdraw_witness.wtns
	cd circuits/reward_withdraw_cpp && ./reward_withdraw ../reward_withdraw_input.json ../reward_withdraw_witness.wtns

	# Generate proofs
	cd circuits && snarkjs groth16 prove coin_withdraw_0001.zkey coin_withdraw_witness.wtns coin_withdraw_proof.json coin_withdraw_public.json
	cd circuits && snarkjs groth16 prove reward_withdraw_0001.zkey reward_withdraw_witness.wtns reward_withdraw_proof.json reward_withdraw_public.json

	# Verify a proof on local machine
	cd circuits && snarkjs groth16 verify coin_withdraw_verif_key.json coin_withdraw_public.json coin_withdraw_proof.json
	cd circuits && snarkjs groth16 verify reward_withdraw_verif_key.json reward_withdraw_public.json reward_withdraw_proof.json
	
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
	cd circuits && snarkjs zkey contribute coin_withdraw_0000.zkey coin_withdraw_0001.zkey --name="1st Contributor Name" -v

circuits/reward_withdraw_0001.zkey: circuits/reward_withdraw.r1cs circuits/pot12_final.ptau
	cd circuits && snarkjs groth16 setup reward_withdraw.r1cs pot12_final.ptau reward_withdraw_0000.zkey
	cd circuits && snarkjs zkey contribute reward_withdraw_0000.zkey reward_withdraw_0001.zkey --name="1st Contributor Name" -v

src/CoinWithdrawVerifier.sol: circuits/coin_withdraw_0001.zkey
	cd circuits && snarkjs zkey export solidityverifier coin_withdraw_0001.zkey ../src/CoinWithdrawVerifier.sol

src/RewardWithdrawVerifier.sol: circuits/reward_withdraw_0001.zkey
	cd circuits && snarkjs zkey export solidityverifier reward_withdraw_0001.zkey ../src/RewardWithdrawVerifier.sol

circuits/pot12_final.ptau:
	# Generate Powers of Tau params
	cd circuits && snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
	cd circuits && snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v
	cd circuits && snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

clean:
	rm -rf src/Verifier.sol test/Verifier.t.sol
	cd circuits && rm -rf *.sym *.r1cs *.json *.zkey *.ptau *.wtns calldata coin_withdraw_js/ coin_withdraw_cpp/