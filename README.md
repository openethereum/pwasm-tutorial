## Tutorial Prerequisites
There is a list of all tools and dependencies required for this tutorial.

### Rust
[rustup](https://github.com/rust-lang-nursery/rustup.rs#installation) is the easiest way to install Rust toolchains. Rust nightly toolchain is required since our contracts require some unstable features:

```bash
rustup install nightly
```

Also we need to install `wasm32-unknown-unknown` to compile contract to Wasm:
```bash
rustup target add wasm32-unknown-unknown
```

### Parity wasm-build
[wasm-build](https://github.com/paritytech/wasm-utils#build-tools-for-cargo) takes the raw `.wasm` file produced by Rust compiler and packs it to the form of valid contract.
```
cargo install pwasm-utils
```

### Parity
Follow this guide https://github.com/paritytech/parity/wiki/Setup. You'll need Parity version **1.9.4** or later.

### Web3.js
We'll be using `Web3.js` to connect to the Parity node. Change dir to the root `pwasm-tutorial` and run [npm](https://nodejs.org/en/) to install `Web3.js`:
```
npm install
```

### Tutorial source code
We provide a full source code for each step in this tutorial under `step-*` directories.

## General structure
Source code: https://github.com/paritytech/pwasm-tutorial/tree/master/step-0

```rust
// Contract doesn't use Rust's standard library
#![no_std]

// `pwasm-ethereum` implements bindings to the runtime
extern crate pwasm_ethereum;

/// Will be described in the next step
#[no_mangle]
pub fn deploy() {
}

/// The call function is the main function of the *deployed* contract
#[no_mangle]
pub fn call() {
    // Send a result pointer to the runtime
    pwasm_ethereum::ret(&b"result"[..]);
}
```
### pwasm-ethereum
[pwasm-ethereum](https://github.com/NikVolf/pwasm-ethereum) is a collection of bindings to interact with ethereum-like network.

## Building
To make sure that everything is setup go to the `step-0` directory and run `./build.sh`

As the result the `pwasm_tutorial_contract.wasm` should be placed under the `step-0/target` directory.

## The constructor
Source code: https://github.com/paritytech/pwasm-tutorial/tree/master/step-1

When deploying a contract we often want to set its ititial storage values (e.g. `totalSupply` if it's a token contact). To address this problem we are exporting another function "deploy" which executes only once on contract deployment.

```rust
// This contract will return the address from which it was deployed
#![no_std]

extern crate pwasm_ethereum;
extern crate parity_hash;

use parity_hash::H256;

// The "deploy" will be executed only once on deployment but will not be stored on the blockchain
#[no_mangle]
pub fn deploy() {
    // Lets set the sender address to the contract storage at address "0"
    pwasm_ethereum::write(&H256::zero().into(), &H256::from(pwasm_ethereum::sender()).into());
    // Note we should't write any result into the call descriptor in deploy.
}

// The following code will be stored on the blockchain.
#[no_mangle]
pub fn call() {
    // Will read the address of the deployer which we wrote to the storage on the deploy stage
    let owner = pwasm_ethereum::read(&H256::zero().into());
    // Send a result pointer to the runtime
    pwasm_ethereum::ret(owner.as_ref());
}
```

## Contract ABI declaration
Source code: https://github.com/paritytech/pwasm-tutorial/tree/master/step-2

Let's implement a simple [ERC-20](https://en.wikipedia.org/wiki/ERC20) token contract.

```rust
// ...

pub mod token {
    use pwasm_ethereum;
    use pwasm_std::*;
    use pwasm_std::hash::H256;
    use bigint::U256;

    // eth_abi is a procedural macros https://doc.rust-lang.org/book/first-edition/procedural-macros.html
    use pwasm_abi_derive::eth_abi;

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
            pwasm_ethereum::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
        }

        fn totalSupply(&mut self) -> U256 {
            pwasm_ethereum::read(&TOTAL_SUPPLY_KEY).into()
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = token::TokenEndpoint::new(token::TokenContractInstance{});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = token::TokenEndpoint::new(token::TokenContractInstance{});
    //
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

```
`token::TokenContract` is the interface definition of the contract.
`pwasm_abi_derive::eth_abi` is a [procedural macros](https://doc.rust-lang.org/book/first-edition/procedural-macros.html) uses a trait `token::TokenContract` to generate decoder (`TokenEndpoint`) for payload in Solidity ABI format. `TokenEndpoint` implements an `EndpointInterface` trait:

```rust
/// Endpoint interface for contracts
pub trait EndpointInterface {
	/// Dispatch payload for regular method
	fn dispatch(&mut self, payload: &[u8]) -> Vec<u8>;

	/// Dispatch constructor payload
	fn dispatch_ctor(&mut self, payload: &[u8]);
}
```

The `dispatch` expects `payload` and returns a result in the format defined in [Solidity ABI spec](http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding). It maps payload to the corresponding method of the `token::TokenContract` implementation. The `dispatch_ctor` maps payload only to the `TokenContract::constructor` and returns no result.

A compete implementation of ERC20 can be found here https://github.com/paritytech/pwasm-token-example.

### pwasm-std
[pwasm-std](https://paritytech.github.io/pwasm-std/pwasm_std/) is the lightweight standard library. It implements common data structures, conversion utils and provides bindings to the runtime.

## Make calls to other contracts
Source code: https://github.com/paritytech/pwasm-tutorial/tree/master/step-3

In order to make calls to our `TokenContract` we need to generate the payload `TokenEndpoint::dispatch()` expects. So `pwasm_abi_derive::eth_abi` can generate an implementation of `TokenContract` which will prepare payload for each method.

```rust
#[eth_abi(TokenEndpoint, TokenClient)]
pub trait TokenContract {
    /// The constructor
    fn constructor(&mut self, _total_supply: U256);
    /// Total amount of tokens
    #[constant] // #[constant] hint affect the resulting JSON abi. It sets "constant": true prore
    fn totalSupply(&mut self) -> U256;
}
```

We've added a second argument `TokenClient` to the `eth_abi` macro, so this way we ask to generate a client implementation for `TokenContract` trait and name it as `TokenClient`. Let's suppose we've deployed a token contract on `0x7BA4324585CB5597adC283024819254345CD7C62` address. That's how we can make calls to it.

```rust
extern pwasm_ethereum;
extern pwasm_std;

use token::TokenClient;
use pwasm_std::hash::Address;

let token = TokenClient::new(Address::from("0x7BA4324585CB5597adC283024819254345CD7C62"));
let tokenSupply = token.totalSupply();
```

`token.totalSupply()` will execute `pwasm_ethereum::call(Address::from("0x7BA4324585CB5597adC283024819254345CD7C62"), payload)` with `address` and `payload` generated according to `totalSupply()` signature. Optionally it's possible to set a `value` (in Wei) to transfer with the call and set a `gas` limit.

```rust
let token = TokenClient::new(Address::from("0x7BA4324585CB5597adC283024819254345CD7C62"))
	.value(10000000.into()) // send a value with the call
	.gas(21000); // set a gas limit
let tokenSupply = token.totalSupply();
```

If you move to `step-3` directory and run `cargo build --release --target wasm32-unknown-unknown` you will find a `TokenContract.json` in the `target/json` generated from `TokenContract` trait with the following content:

```json
[
  {
    "type": "function",
    "name": "totalSupply",
    "inputs": [],
    "outputs": [
      {
        "name": "returnValue",
        "type": "uint256"
      }
    ],
    "constant": true
  },
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "_total_supply",
        "type": "uint256"
      }
    ]
  }
]
```

JSON above is an ABI definition which can be used along with Web.js to run transactions and calls to contract:

```javascript
var Web3 = require("web3");
var fs = require("fs");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
var abi = JSON.parse(fs.readFileSync("./target/TokenContract.json"));
var TokenContract = new web3.eth.Contract(abi, "0x7BA4324585CB5597adC283024819254345CD7C62", { from: web3.eth.defaultAccount });
var totalSupply = TokenContract.methods.totalSupply().call().then(console.log);
```

### Events
Source code: https://github.com/paritytech/pwasm-tutorial/tree/master/step-4

Events allow the convenient usage of the EVM logging facilities, which in turn can be used to “call” JavaScript callbacks in the user interface of a dapp, which listen for these events.

Let's implement the `transfer` method for our ERC-20 contract. `step-4` directory contains the complete implementation.

```rust
pub mod token {

    #[eth_abi(TokenEndpoint, TokenClient)]
    pub trait TokenContract {
        /// The constructor
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of tokens
        #[constant]
        fn totalSupply(&mut self) -> U256;
        /// What is the balance of a particular account?
        #[constant]
        fn balanceOf(&mut self, _owner: Address) -> U256;
        /// Transfer the balance from owner's account to another account
        fn transfer(&mut self, _to: Address, _amount: U256) -> bool;
        /// Event declaration
        #[event]
        fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, _value: U256);
    }

    pub struct TokenContractInstance;

    impl TokenContract for TokenContractInstance {
        fn constructor(&mut self, total_supply: U256) {
            // ...
        }

        fn totalSupply(&mut self) -> U256 {
            // ...
        }

        fn balanceOf(&mut self, owner: Address) -> U256 {
            read_balance_of(&owner)
        }

        fn transfer(&mut self, to: Address, amount: U256) -> bool {
            let sender = pwasm_ethereum::sender();
            let senderBalance = read_balance_of(&sender);
            let recipientBalance = read_balance_of(&to);
            if amount == 0.into() || senderBalance < amount {
                false
            } else {
                let new_sender_balance = senderBalance - amount;
                let new_recipient_balance = recipientBalance + amount;
                pwasm_ethereum::write(&balance_key(&sender), &new_sender_balance.into());
                pwasm_ethereum::write(&balance_key(&to), &new_recipient_balance.into());
                self.Transfer(sender, to, amount);
                true
            }
        }
    }

    // Reads balance by address
    fn read_balance_of(owner: &Address) -> U256 {
        pwasm_ethereum::read(&balance_key(owner)).into()
    }

    // Generates a balance key for some address.
    // Used to map balances with their owners.
    fn balance_key(address: &Address) -> H256 {
        let mut key = H256::from(address);
        key[0] = 1; // just a naiive "namespace";
        key
    }
}
```

Events are declared as part of a contract trait definition. Arguments which start with the "indexed_" prefix are considered as "topics", other arguments are data associated with an event.

```rust
#[eth_abi(TokenEndpoint, TokenClient)]
pub trait TokenContract {
    fn transfer(&mut self, _to: Address, _amount: U256) -> bool;
    #[event]
    fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, _value: U256);
}

fn transfer(&mut self, to: Address, amount: U256) -> bool {
    let sender = pwasm_ethereum::sender();
    let senderBalance = read_balance_of(&sender);
    let recipientBalance = read_balance_of(&to);
    if amount == 0.into() || senderBalance < amount {
        false
    } else {
        let new_sender_balance = senderBalance - amount;
        let new_recipient_balance = recipientBalance + amount;
        pwasm_ethereum::write(&balance_key(&sender), &new_sender_balance.into());
        pwasm_ethereum::write(&balance_key(&to), &new_recipient_balance.into());
        self.Transfer(sender, to, amount);
        true
    }
}
```

Topics are useful to filter events produced by contract. In following example we use Web3.js to subscribe to the `Transfer` events of deployed `TokenContract`.
```javascript
var Web3 = require("web3");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
var abi = JSON.parse(fs.readFileSync("./target/TokenContract.json"));
var TokenContract = new web3.eth.Contract(abi, "0x7BA4324585CB5597adC283024819254345CD7C62", { from: web3.eth.defaultAccount });

// Subscribe to the Transfer event
TokenContract.events.Transfer({
    from: "0x7BA4324585CB5597adC283024819254345CD7C62" // Filter transactions by sender
}, function (err, event) {
    console.log(event);
});
```

## Deploy
Starting from version **1.9.4** Parity includes support for running Wasm contracts. Wasm support isn't enabled by default and needs to be specified in the "chainspec" file. `wasmActivationTransition` param sets a block number Wasm support should be activated. This is a sample "development chain" spec with Wasm enabled (based on https://paritytech.github.io/wiki/Private-development-chain):

[Source](https://github.com/paritytech/pwasm-tutorial/tree/master/wasm-dev-chain.json)
```json
{
    "name": "DevelopmentChain",
    "engine": {
        "instantSeal": null
    },
    "params": {
        "wasmActivationTransition": "0x01",
        "gasLimitBoundDivisor": "0x0400",
        "accountStartNonce": "0x0",
        "maximumExtraDataSize": "0x20",
        "minGasLimit": "0x1388",
        "networkID" : "0x11"
    },
    "genesis": {
        "seal": {
            "generic": "0x0"
        },
        "difficulty": "0x20000",
        "author": "0x0000000000000000000000000000000000000000",
        "timestamp": "0x00",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "extraData": "0x",
        "gasLimit": "0x5B8D80"
    },
    "accounts": {
        "0000000000000000000000000000000000000001": { "balance": "1", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
        "0000000000000000000000000000000000000002": { "balance": "1", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
        "0000000000000000000000000000000000000003": { "balance": "1", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
        "0000000000000000000000000000000000000004": { "balance": "1", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
        "0x004ec07d2329997267ec62b4166639513386f32e": { "balance": "1606938044258990275541962092341162602522202993782792835301376" }
    }
}

```
Run Parity:
```bash
parity --chain ./wasm-dev-chain.json --jsonrpc-apis=all
```
Let it run in a separate terminal window.

Among with other things we've added an account `0x004ec07d2329997267ec62b4166639513386f32e` with some ETH to `wasm-dev-chain.json` on which behalf we'll run transactions. Now we need to add this account to the local keychain:

```bash
curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromPhrase","params":["user", "user"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8545
```
Should output something like:
```json
{"jsonrpc":"2.0","result":"0x004ec07d2329997267ec62b4166639513386f32e","id":0}
```

Now cd to `step-4` and build the contract:
```bash
./build.sh
```
It should produce 2 files we need:
- a compiled Wasm binary `./target/pwasm_tutorial_contract.wasm`
- an ABI file: `./target/json/TokenContract.json`

At this point we can use Web.js to connect to the Parity node and deploy Wasm `pwasm_tutorial_contract.wasm`. Run the following code in `node` console:

```javascript
var Web3 = require("web3");
var fs = require("fs");
// Connect to our local node
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
// Setup default account
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
// read JSON ABI
var abi = JSON.parse(fs.readFileSync("./target/json/TokenContract.json"));
// convert Wasm binary to hex format
var codeHex = '0x' + fs.readFileSync("./target/pwasm_tutorial_contract.wasm").toString('hex');

var TokenContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount });

var TokenDeployTransaction = TokenContract.deploy({data: codeHex, arguments: [10000000]});

// Will create TokenContract with `totalSupply` = 10000000 and print a result
web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user").then(() => TokenDeployTransaction.estimateGas()).then(gas => TokenDeployTransaction.send({gasLimit: gas, from: web3.eth.defaultAccount})).then(contract => { console.log("Address of new contract: " + contract.options.address); TokenContract = contract; }).catch(err => console.log(err));
```
Now we're able transfer some tokens:
```javascript
web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user").then(() => TokenContract.methods.transfer("0x7BA4324585CB5597adC283024819254345CD7C62", 200).send()).then(console.log).catch(console.log);
```

And check balances:
```javascript
// Check balance of recipient. Should print 200
TokenContract.methods.balanceOf("0x7BA4324585CB5597adC283024819254345CD7C62").call().then(console.log).catch(console.log);

// Check balance of sender (owner of contract). Should print 10000000 - 200 = 9999800
TokenContract.methods.balanceOf(web3.eth.defaultAccount).call().then(console.log).catch(console.log);
```

## Testing
[pwasm-test](https://github.com/paritytech/pwasm-test) makes it easy to test a contract logic. It allows to emulate the blockchain state and mock any [pwasm-ethereum](#pwasm-ethereum) call.

By default our contracts built with a `#![no_std]`, but `rust test` need the Rust stdlib for treading and i/o. Thus, in order to run tests we've added a following feature gate in [Cargo.toml](https://github.com/paritytech/pwasm-tutorial/tree/master/step-5):

```
[features]
std = ["pwasm-std/std", "pwasm-ethereum/std"]
```
Now you can `cd step-5` and `cargo test --features std` should pass.

Take a look https://github.com/paritytech/pwasm-tutorial/blob/master/step-5/src/sample.rs#L116-L161 to see an example how to test a `transfer` method of our token contract.

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;
    use super::*;
    use self::pwasm_test::{ext_reset, ext_get};
    use parity_hash::Address;
    use token::TokenContract;

    #[test]
    fn should_succeed_transfering_1000_from_owner_to_another_address() {
        let mut contract = token::TokenContractInstance{};
        let owner_address = Address::from("0xea674fdde714fd979de3edf0f56aa9716b898ec8");
        let sam_address = Address::from("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1");
        // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
        // so `pwasm_ethereum::sender()` in TokenContract::constructor() will return that `owner_address`
        ext_reset(|e| e.sender(owner_address.clone()));
        let total_supply = 10000.into();
        contract.constructor(total_supply);
        assert_eq!(contract.balanceOf(owner_address), total_supply);
        assert_eq!(contract.transfer(sam_address, 1000.into()), true);
        assert_eq!(contract.balanceOf(owner_address), 9000.into());
        assert_eq!(contract.balanceOf(sam_address), 1000.into());
        // 1 log entry should be created
        assert_eq!(ext_get().logs().len(), 1);
    }
}
```

[Here](https://github.com/paritytech/pwasm-test/tree/master/tests) you can find more examples on how to:
- [mock calls](https://github.com/paritytech/pwasm-test/blob/master/tests/calls.rs) to other contracts
- [read event logs created by contract](https://github.com/paritytech/pwasm-test/blob/master/tests/log.rs)
- [init contract with storage](https://github.com/paritytech/pwasm-test/blob/master/tests/storage_read.rs).

More testing examples:
https://github.com/paritytech/pwasm-token-example/blob/master/contract/src/lib.rs#L194

In order to test interaction between contracts we're able to mock callee contract client. See comprehensive here:
https://github.com/paritytech/pwasm-repo-contract/blob/master/contract/src/lib.rs#L453
