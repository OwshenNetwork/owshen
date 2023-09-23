.PHONY = test clean

bindings: contracts/src/*.sol
	rm -rf bindings
	cd contracts && forge bind --bindings-path ../bindings --root . --crate-name bindings

test: bindings
	cd contracts && make build
	cargo test -- --test-threads 1

clean:
	rm -rf bindings
	cd contracts && make clean