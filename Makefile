t:
	cargo test -p pallet-poe

b:
	cargo build --release

br:
	cargo build --release && ./target/release/node-template --dev