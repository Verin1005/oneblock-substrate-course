t:
	cargo test -p pallet-example

b:
	cargo build --release

br:
	cargo build --release && ./target/release/node-template --dev