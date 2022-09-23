t:
	cargo test -p pallet-example

b:
	cargo build --release

br:
	cargo build --release && ./target/release/node-template --dev

r:
	cargo run --release -- --dev --enable-offchain-indexing true

fr:
	cd substrate-front-end-template && yarn start