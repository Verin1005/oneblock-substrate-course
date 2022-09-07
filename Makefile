t:
	cargo test -p pallet-kitties
tt:
	RUST_BACKTRACE=1 cargo test -p pallet-kitties

b:
	cargo build -p pallet-kitties

br:
	cargo build --release && ./target/release/node-template --dev