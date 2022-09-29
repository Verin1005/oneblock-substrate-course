t:
	cargo test -p pallet-example

b_contract:
	cd erc20 && cargo +stable contract build
t_contract:
	cd erc20 && cargo +stable test

r_node:
	cd substrate-contracts-node && cargo run --release -- --dev

br:
	cargo build --release && ./target/release/node-template --dev
