.PHONY = test clean

test:
	cd contracts && make build
	cd contracts && forge bind --bindings-path ../bindings --root . --crate-name bindings
	cargo test -- --test-threads 1

clean:
	rm -rf bindings
	cd contracts && make clean