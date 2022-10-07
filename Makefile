t:
	cargo test -p pallet-poe

b:
	cargo build --release

br:
	cargo build --release && ./target/release/node-template --dev

bench-b:
	cargo build --release --features runtime-benchmarks

cp_template:
	cp ../substrate/.maintain/frame-weight-template.hbs ./.maintain/frame-weight-template.hbs

bench: bench-b
	./target/release/node-template benchmark pallet \
		--chain=dev \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=pallet_poe \
		--extrinsic="*" \
		--steps=20 \
		--repeat=10 \
		--template=./.maintain/frame-weight-template.hbs \
		--output=./pallets/poe/src/weights.rs