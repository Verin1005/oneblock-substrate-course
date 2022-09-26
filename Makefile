t:
	cargo test -p pallet-example

b:
	cargo build --release

r-node:
	cd substrate-contract-node && cargo run --release -- --dev

br:
	cargo build --release && ./target/release/node-template --dev
