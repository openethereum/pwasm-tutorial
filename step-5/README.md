## Description
The step-by-step project describes how to write a contract in Rust and compile in to Webassembly.

## Build
Make sure you've installed [required tools](https://github.com/paritytech/pwasm-tutorial/blob/master/README.md#tutorial-prerequisites)
```
./build.sh
```
As a result the `pwasm_tutorial_contract.wasm` should be created under the `step-5/target/wasm32-unknown-unknown/release/` directory.

## Deploy
See https://github.com/fckt/pwasm-tutorial#deploy

## Test
```
cargo test --features std
```
