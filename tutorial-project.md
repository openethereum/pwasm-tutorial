## Tutorial Prerequisites
There is a list of all tools and dependencies required for this tutorial.

### Rust
The easiest way to install is using [rustup](https://github.com/rust-lang-nursery/rustup.rs#installation). We need to install rustc nightly since our contracts require some unstable features:
```
rustup install nightly
```

Also we need to install wasm32-unknown-unknown to compile contract to Wasm:
```
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Parity wasm-build
[wasm-build](https://github.com/paritytech/wasm-utils#build-tools-for-cargo) takes the raw `.wasm` file produced by `rustc` compiler, pass it through optimization and packs it into the final Wasm-contract ready for deployment on the chain.
```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```

### Parity
https://github.com/paritytech/parity
Parity 1.10 // TODO: installation

## General structure

## Building

## Contract ABI declaration

### Calls

### Transactions

### Events

## Testing
