.PHONY: build

ENTROPY = 1234

build:  proof_of_burn_circuits/mpt_last_cpp/ proof_of_burn_circuits/mpt_last_0001.zkey proof_of_burn_circuits/mpt_last_verif_key.json src/MptLastVerifier.sol proof_of_burn_circuits/mpt_path_cpp/ proof_of_burn_circuits/mpt_path_0001.zkey proof_of_burn_circuits/mpt_path_verif_key.json src/MptPathVerifier.sol proof_of_burn_circuits/spend_cpp/ proof_of_burn_circuits/spend_0001.zkey proof_of_burn_circuits/spend_verif_key.json src/SpendVerifier.sol 

proof_of_burn_circuits/mpt_last_verif_key.json: proof_of_burn_circuits/mpt_last_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export verificationkey mpt_last_0001.zkey mpt_last_verif_key.json

proof_of_burn_circuits/mpt_last.r1cs proof_of_burn_circuits/mpt_last_cpp/: proof_of_burn_circuits/mpt_last.circom
	cd proof_of_burn_circuits && circom mpt_last.circom --r1cs --wasm --sym --c

	mv proof_of_burn_circuits/mpt_last_cpp/main.cpp proof_of_burn_circuits/mpt_last_cpp/main.cpp.tmp
	python3 scripts/spit_output.py < proof_of_burn_circuits/mpt_last_cpp/main.cpp.tmp > proof_of_burn_circuits/mpt_last_cpp/main.cpp
	rm proof_of_burn_circuits/mpt_last_cpp/main.cpp.tmp

	cd proof_of_burn_circuits/mpt_last_cpp && make

proof_of_burn_circuits/mpt_last_0001.zkey: proof_of_burn_circuits/mpt_last.r1cs
	cd proof_of_burn_circuits && snarkjs groth16 setup mpt_last.r1cs ../../contracts/circuits/pot_final.ptau mpt_last_0000.zkey
	cd proof_of_burn_circuits && snarkjs zkey contribute mpt_last_0000.zkey mpt_last_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

src/MptLastVerifier.sol: proof_of_burn_circuits/mpt_last_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export solidityverifier mpt_last_0001.zkey ../../contracts/src/MptLastVerifier.sol
	sed -i 's/Groth16Verifier/MptLastVerifier/' ../contracts/src/MptLastVerifier.sol

proof_of_burn_circuits/mpt_path_verif_key.json: proof_of_burn_circuits/mpt_path_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export verificationkey mpt_path_0001.zkey mpt_path_verif_key.json

proof_of_burn_circuits/mpt_path.r1cs proof_of_burn_circuits/mpt_path_cpp/: proof_of_burn_circuits/mpt_path.circom
	cd proof_of_burn_circuits && circom mpt_path.circom --r1cs --wasm --sym --c

	mv proof_of_burn_circuits/mpt_path_cpp/main.cpp proof_of_burn_circuits/mpt_path_cpp/main.cpp.tmp
	python3 scripts/spit_output.py < proof_of_burn_circuits/mpt_path_cpp/main.cpp.tmp > proof_of_burn_circuits/mpt_path_cpp/main.cpp
	rm proof_of_burn_circuits/mpt_path_cpp/main.cpp.tmp

	cd proof_of_burn_circuits/mpt_path_cpp && make

proof_of_burn_circuits/mpt_path_0001.zkey: proof_of_burn_circuits/mpt_path.r1cs
	cd proof_of_burn_circuits && snarkjs groth16 setup mpt_path.r1cs ../../contracts/circuits/pot_final.ptau mpt_path_0000.zkey
	cd proof_of_burn_circuits && snarkjs zkey contribute mpt_path_0000.zkey mpt_path_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

src/MptPathVerifier.sol: proof_of_burn_circuits/mpt_path_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export solidityverifier mpt_path_0001.zkey ../../contracts/src/MptPathVerifier.sol
	sed -i 's/Groth16Verifier/MptPathVerifier/' ../contracts/src/MptPathVerifier.sol

proof_of_burn_circuits/spend_verif_key.json: proof_of_burn_circuits/spend_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export verificationkey spend_0001.zkey spend_verif_key.json

proof_of_burn_circuits/spend.r1cs proof_of_burn_circuits/spend_cpp/: proof_of_burn_circuits/spend.circom
	cd proof_of_burn_circuits && circom spend.circom --r1cs --wasm --sym --c
	cd proof_of_burn_circuits/spend_cpp && make

proof_of_burn_circuits/spend_0001.zkey: proof_of_burn_circuits/spend.r1cs
	cd proof_of_burn_circuits && snarkjs groth16 setup spend.r1cs ../../contracts/circuits/pot_final.ptau spend_0000.zkey
	cd proof_of_burn_circuits && snarkjs zkey contribute spend_0000.zkey spend_0001.zkey --name="1st Contributor Name" --entropy=${ENTROPY} -v

src/SpendVerifier.sol: proof_of_burn_circuits/spend_0001.zkey
	cd proof_of_burn_circuits && snarkjs zkey export solidityverifier spend_0001.zkey ../../contracts/src/SpendVerifier.sol
	sed -i 's/Groth16Verifier/SpendVerifier/' ../contracts/src/SpendVerifier.sol

clean:
	cd proof_of_burn_circuits && rm -rf *.sym *.r1cs *.json *.zkey *.wtns mpt_last_js/ mpt_last_cpp/ mpt_last_calldata mpt_path_js/ mpt_path_cpp/ mpt_path_calldata spend_js/ spend_cpp/ spend_calldata