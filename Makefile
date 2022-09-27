t:
	cargo test -p pallet-example

env_stable:
	rustup default stable
env_nightly:
	rustup default nightly
b_contract: env_stable
	cd erc20 && cargo contract build
t_contract: env_stable
	cd erc20 && cargo test

r_node:
	cd substrate-contracts-node && cargo run --release -- --dev

br:
	cargo build --release && ./target/release/node-template --dev
