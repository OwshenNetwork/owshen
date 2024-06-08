.PHONY: test clean install windows

rapidsnark/package/bin/prover:
	cd rapidsnark && git submodule init
	cd rapidsnark && git submodule update
	cd rapidsnark && ./build_gmp.sh host
	cd rapidsnark && mkdir -p build_prover
	cd rapidsnark && cd build_prover && cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package
	cd rapidsnark && cd build_prover && make -j4 && make install

bindings: contracts/src/*.sol
	rm -rf bindings
	cd circuits && make build
	cd contracts && make build
	cd contracts && forge bind --bindings-path ../bindings --root . --crate-name bindings

install: bindings
	cargo install --path .

test: bindings
	cargo test -- --test-threads 1

clean:
	cd rapidsnark && make clean
	rm -rf bindings
	cd contracts && make clean
	cd circuits && make clean

assets:
	rm -rf assets/*
	mkdir assets
	mkdir assets/zk
	mkdir assets/witness
	mkdir assets/bin
	cp -r contracts/circuits/coin_withdraw_js/* assets/witness
	cp -r contracts/circuits/coin_withdraw_0001.zkey assets/zk
	cp -r rapidsnark/package/bin/prover assets/bin
	cp -r contracts/circuits/coin_withdraw_cpp/coin_withdraw assets/bin
	cp -r contracts/circuits/coin_withdraw_cpp/coin_withdraw.dat assets/bin
	cp -r owshen-genesis.dat assets/bin
	cp -r client/build/static assets
	cp -r client/build/static assets
	cp -r client/build/static assets
	cp -r client/build/*.html assets
	cp -r client/build/*.json assets
	cp -r client/build/*.txt assets
	cp -r client/build/*.ico assets
	cp -r networks assets
	
