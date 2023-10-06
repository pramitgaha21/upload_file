rm -rf wasm_files
mkdir wasm_files

# building storage
cargo build --target wasm32-unknown-unknown --release --package storage
cargo test --package storage

ic-wasm target/wasm32-unknown-unknown/release/storage.wasm -o wasm_files/storage.wasm metadata candid:service -v public -f src/storage/storage.did
ic-wasm wasm_files/storage.wasm -o wasm_files/storage.wasm shrink

# building scaler
cargo build --target wasm32-unknown-unknown --release --package scaler
candid-extractor target/wasm32-unknown-unknown/release/scaler.wasm > src/scaler/scaler.did || true

ic-wasm target/wasm32-unknown-unknown/release/scaler.wasm -o wasm_files/scaler.wasm metadata candid:service -v public -f src/scaler/scaler.did
ic-wasm wasm_files/scaler.wasm -o wasm_files/scaler.wasm shrink
