RUSTFLAGS='-C target-feature=+simd128,+atomics,+bulk-memory,+mutable-globals -Clink-arg=--max-memory=4294967296' \
  cargo build --target wasm32-unknown-unknown -Z build-std=std,panic_abort --release
cp target/wasm32-unknown-unknown/release/ld50.wasm web_build/wasm.wasm
