## Tutorial Prerequisites
There is a list of all tools and dependencies required for this tutorial.

### Rust
[rustup](https://github.com/rust-lang-nursery/rustup.rs#installation) is the easiest way to install rust toolchain. We need to install rustc nightly since our contracts require some unstable features:
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

### Tutorial source code
We provide a full source code for each step in this tutorial under step-* directories.

## General structure

```rust
// Contract doesn't use Rust's standard library
#![no_std]
#![feature(wasm_import_memory)]
#![wasm_import_memory]

// `pwasm-std` is the lightweight implementation of a standard library
// It implements common data structures and provides bindings to the runtime
extern crate pwasm_std;

/// Will be described in the next step
#[no_mangle]
pub fn deploy(desc: *mut u8) {
}

/// The call function is the main function of the *deployed* contract
/// Function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    // pwasm_std::parse_args splits the call descriptor into arguments and result pointers
    let (_args, result) = unsafe { pwasm_std::parse_args(desc) };
    // result.done writes the result vector to the call descriptor.
    result.done(b"result".to_vec());
}

```
## Building
To make sure that everything is setup go to the `step-0` directory and run:
```
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target pwasm_tutorial_contract
```
As the result the `pwasm_tutorial_contract.wasm` should be placed under the `step-0/target` directory.

## The constructor
When deploying a contract we often want to setup its initial storage. To solve this problem we are exporting another function "deploy" which executes only once on contract deployment.

https://github.com/fckt/pwasm-tutorial-project/tree/step-1/src
```rust
/// This contract will return the address from which it was deployed

#![no_std]
#![feature(wasm_import_memory)]
#![wasm_import_memory]

extern crate pwasm_std;

extern crate pwasm_ethereum;

use pwasm_std::hash::H256;

// The "deploy" will be executed only once on deployment but will not be stored on the blockchain
#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    // Lets set the sender address to the contract storage at address "0"
    pwasm_ethereum::storage::write(&H256::zero().into(), &H256::from(pwasm_ethereum::ext::sender()).into());
    // Note we should't write any result into the call descriptor in deploy.
}

// The following code will be stored on the blockchain.
#[no_mangle]
pub fn call(desc: *mut u8) {
    // pwasm_std::parse_args splits the call descriptor into arguments and result pointers
    let (_args, result) = unsafe { pwasm_std::parse_args(desc) };
    // Will read the address of the deployer which we wrote to the storage on the deploy stage
    let owner = pwasm_ethereum::storage::read(&H256::zero().into());
    // result.done() writes the result vector to the call descriptor.
    result.done(owner.to_vec());
}
```

## Contract ABI declaration


TokenContract is an interface definition of a contract.
eth_abi macro parses an interface (trait) definition of a contact and generates
two structs: `Endpoint` and `Client`.

`Endpoint` is an entry point for contract calls. Endpoint implements the EndpointInterface with 2 methods
eth_abi macro generates a table of Method IDs corresponding with every method signature defined in the trait
and defines it statically in the generated code.
Scroll down at "pub fn call(desc: *mut u8)" to see how.
Endpoint instantiates with a struct TokenContractInstance which implements the TokenContract trait definition.

`Client` is a struct which is useful for call generation to a deployed contract. For example:
```rust
	let mut client = Client::new(contactAddress);
	let balance = client
		.value(someValue) // you can attach some value for a call optionally
		.balanceOf(someAddress);
```
Will generate a Solidity-compatible call for the contract, deployed on `contactAddress`.
Then it invokes pwasm_std::ext::call on `contactAddress` and returns the result.

## Building

### Calls

### Transactions

### Events

## Testing
