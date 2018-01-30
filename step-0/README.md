## Description
The step-by-step project describes how to write a contract in Rust and compile in to Webassembly.

## Build
```
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target pwasm_tutorial_contract
```
