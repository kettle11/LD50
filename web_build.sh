RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -Clink-arg=--max-memory=4294967296' \
  cargo build --target wasm32-unknown-unknown -Z build-std=std,panic_abort --release
cp target/wasm32-unknown-unknown/release/rapier_test.wasm web_build/wasm.wasm
