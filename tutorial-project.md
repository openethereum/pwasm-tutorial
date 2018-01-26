## Tutorial Prerequisites
There is a list of all tools and dependencies required for this tutorial.

### Rust
[rustup](https://github.com/rust-lang-nursery/rustup.rs#installation) is the easiest way to install Rust toolchains. Rust nightly toolchain is required since our contracts require some unstable features:
```
rustup install nightly
```

Also we need to install `wasm32-unknown-unknown` to compile contract to Wasm:
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
We provide a full source code for each step in this tutorial under `step-*` directories.

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
Let's implement a simple [ERC-20](https://en.wikipedia.org/wiki/ERC20) token contract.

```rust
#![no_std]
#![feature(alloc)]
#![feature(proc_macro)]
#![feature(wasm_import_memory)]
#![wasm_import_memory]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate alloc;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
/// Bigint used for 256-bit arithmetic
extern crate bigint;

mod token {
    use pwasm_ethereum::{storage};
    use pwasm_std::hash::{H256};
    use bigint::U256;

	// eth_abi is a procedural macros https://doc.rust-lang.org/book/first-edition/procedural-macros.html
    use pwasm_abi_derive::eth_abi;
    use alloc::Vec;

    static TOTAL_SUPPLY_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

    #[eth_abi(TokenEndpoint)]
    pub trait TokenContract {
		/// The constructor
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of tokens
        fn totalSupply(&mut self) -> U256;
    }

    pub struct TokenContractInstance;

    impl TokenContract for TokenContractInstance {
        fn constructor(&mut self, total_supply: U256) {
            // Set up the total supply for the token
            storage::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
        }

        fn totalSupply(&mut self) -> U256 {
            storage::read(&TOTAL_SUPPLY_KEY).into()
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

/// The main function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = token::TokenEndpoint::new(token::TokenContractInstance{});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = token::TokenEndpoint::new(token::TokenContractInstance{});
    endpoint.dispatch_ctor(&args);
}
```
`token::TokenContract` is the interface definition of the contract.
`pwasm_abi_derive::eth_abi` is a [procedural macros](https://doc.rust-lang.org/book/first-edition/procedural-macros.html) which parses an interface (trait) definition of a contact and generates `TokenEndpoint`. `TokenEndpoint` implements an `EndpointInterface` trait.

```rust
/// Endpoint interface for contracts
pub trait EndpointInterface {
	/// Dispatch payload for regular method
	fn dispatch(&mut self, payload: &[u8]) -> Vec<u8>;

	/// Dispatch constructor payload
	fn dispatch_ctor(&mut self, payload: &[u8]);
}
```

The `dispatch` expects `payload` and returns result in format defined in [Solidity ABI spec](http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding). It maps payload to the corresponding method of the `token::TokenContract` implementation. The `dispatch_ctor` maps payload only to the `TokenContract::constructor` and returns no result.

A compete implementation of ERC20 can be found here https://github.com/paritytech/pwasm-token-example.

## Make calls to other contracts
In order to make calls to our `TokenContract` we need to generate the payload `TokenEndpoint` expects. So `pwasm_abi_derive::eth_abi` can generate a client for the contract.

```rust
#[eth_abi(TokenEndpoint, TokenClient)]
    pub trait TokenContract {
		/// The constructor
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of tokens
        fn totalSupply(&mut self) -> U256;
    }
```
We have added a second argument `TokenClient` to the `eth_abi` macro, so this way we ask to generate a client implementation for TokenContract and name it as `TokenClient`. So for example if we've deployed our `TokenContract` on the chain at address `0xe1EDa226759825E236001714bcDc0ca0B21fd800` we can use `TokenClient` as following:

```rust
extern pwasm_ethereum;
extern pwasm_std;

use token::TokenClient;
use pwasm_std::hash::Address;

let token = TokenClient::new(Address::from("0xe1EDa226759825E236001714bcDc0ca0B21fd800"));
let tokenSupply = token.totalSupply();
```

`token.totalSupply()` will execute `pwasm_ethereum::call(Address::from("0xe1EDa226759825E236001714bcDc0ca0B21fd800"), payload)` with address and payload generated for `totalSupply()` signature. Optionally it's possible to set a `value` (in Wei) to transfer with the call and set a `gas` limit.

```rust
let token = TokenClient::new(Address::from("0xe1EDa226759825E236001714bcDc0ca0B21fd800"))
	.value(10000000.into())
	.gas(21000);
let tokenSupply = token.totalSupply();
```

This is an example on how one contract call another:
https://github.com/paritytech/pwasm-repo-contract/blob/50d3acf04e99b33b66773ee84226885d4621d631/contract/src/lib.rs#L262-L273

## Building

### Transactions

### Events

## Testing
