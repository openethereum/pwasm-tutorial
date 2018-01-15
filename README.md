This tutorial cover steps how to compile contract written in Rust to Wasm module, pack it as constructor and deploy on the chain.

## Compilation

First we need `wasm32-unknown-unknown` target to be installed:

```
rustup update
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Also we need to install a build tool from https://github.com/paritytech/wasm-utils which do pack Wasm into contract constructor and optimize size of the final wasm trowing away all code which is unused:

```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```

Now let's compile a sample ERC-20 contract. Clone https://github.com/paritytech/pwasm-token-example somewhere and run:

```
cd pwasm-token-example
cargo build --release --target wasm32-unknown-unknown
```

It will produce a `token.wasm` under `./target/wasm32-unknown-unknown`. Thanks to the  [pwasm-abi crate](https://github.com/paritytech/pwasm-abi) it'll also generate the Solidity ABI for us and save as `TokenContract.json` under `./target/json` directory.

Now run:

```
wasm-build --target wasm32-unknown-unknown ./target token // "token" is the name of *wasm binary to find
```

`wasm-build` will look for compiled Wasm under the `./target/wasm32-unknown-unknown` dir and put the final wasm into the `./target/token.wasm`.

In order to successfully build the final deployable contract Wasm module should contain 2 exports: `call` and `deploy` (see https://github.com/paritytech/pwasm-token-example/blob/master/src/token.rs). The `call` export used to call methods on the deployed contract while `deploy` should contain the initialization code. `wasm-build` optimise contact over `call`, puts its binary code into the static data segment, renames `deploy` export to `call` and makes it return a pointer to and length of that static data segment.

## Deployment

Let's deploy packed `token.wasm` on the chain. For purposes of tutorial we will use a simple PoA configuration. Follow instructions in the https://github.com/paritytech/parity/wiki/Demo-PoA-tutorial. To enable support of Wasm-contracts in our chain, set `"wasm": true` under `params` in the `demo-spec.json`, so the final `demo-spec.json` should look like:
