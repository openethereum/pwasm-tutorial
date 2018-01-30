## Tutorial Prerequisites
There is a list of all tools and dependencies required for this tutorial.

### Rust
[rustup](https://github.com/rust-lang-nursery/rustup.rs#installation) is the easiest way to install Rust toolchains. Rust nightly toolchain is required since our contracts require some unstable features:

```bash
rustup install nightly
```

Also we need to install `wasm32-unknown-unknown` to compile contract to Wasm:

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Parity wasm-build
[wasm-build](https://github.com/paritytech/wasm-utils#build-tools-for-cargo) takes the raw `.wasm` file produced by `rustc` compiler, pass it through optimization and packs it into the final Wasm-contract ready for deployment on the chain.
```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```

### Parity
Follow this guide https://github.com/paritytech/parity/wiki/Setup

### Web3.js
Follow this guide https://github.com/ethereum/web3.js/#installation

### Tutorial source code
We provide a full source code for each step in this tutorial under `step-*` directories.

## General structure
Source code: https://github.com/fckt/pwasm-tutorial/tree/master/step-0

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

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target pwasm_tutorial_contract
```
As the result the `pwasm_tutorial_contract.wasm` should be placed under the `step-0/target` directory.

## The constructor
Source code: https://github.com/fckt/pwasm-tutorial/tree/master/step-1

When deploying a contract we often want to setup its initial storage. To solve this problem we are exporting another function "deploy" which executes only once on contract deployment.

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
    // Lets set the sender address to the contract storage
    pwasm_ethereum::storage::write(&H256::zero().into(), &H256::from(pwasm_ethereum::ext::sender()).into());
    // Note we shouldn't write any result into the call descriptor in deploy.
}

// The following function will be the main function of the deployed contract
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
Source code: https://github.com/fckt/pwasm-tutorial/tree/master/step-2

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
Source code: https://github.com/fckt/pwasm-tutorial/tree/master/step-3

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

We've added a second argument `TokenClient` to the `eth_abi` macro, so this way we ask to generate a client implementation for `TokenContract` trait and name it as `TokenClient`. Let's suppose we've deployed a token contract on `0xe1EDa226759825E236001714bcDc0ca0B21fd800` address. That's how we can make calls to it.

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

That JSON is an ABI definition which can be used along with Web.js to run transactions and calls to contract:

```javascript
var Web3 = require("web3");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
var abi = JSON.parse(fs.readFileSync("./target/TokenContract.json"));
var TokenContract = new web3.eth.Contract(abi, "0xe1EDa226759825E236001714bcDc0ca0B21fd800", { from: web3.eth.defaultAccount });
var totalSupply = TokenContract.methods.totalSupply();
```

### Events
Source code: https://github.com/fckt/pwasm-tutorial/tree/master/master/step-4

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

Events are declared as part of contract trait definition. Arguments which start with the "indexed_" prefix considered as "topics", other arguments are data associated with event.

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
    var TokenContract = new web3.eth.Contract(abi, "0xe1EDa226759825E236001714bcDc0ca0B21fd800", { from: web3.eth.defaultAccount });
    var event = TokenContract.Transfer({valueA: 23} [, additionalFilterObject])

    // Subscribe to the Transfer event
    TokenContract.events.Transfer({
        from: "0xe2fDa626759825E236001714bcDc0ca0B21fd800" // Filter transactions by sender
    }, function (err, event) {
        console.log(event);
    });
```

## Deploy
Starting from version 1.8 Parity includes support for running Wasm contracts. Wasm support isn't enabled by default it should be specified in the "chainspec" file. This is a sample "development chain" spec with Wasm enabled (based on https://paritytech.github.io/wiki/Private-development-chain):

[Source](https://github.com/fckt/pwasm-tutorial/tree/master/step-4/wasm-dev-chain.json)
```
{
    "name": "DevelopmentChain",
    "engine": {
        "instantSeal": null
    },
    "params": {
        "wasm": true,
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
        "0x00a329c0648769a73afac7f9381e08fb43dbea72": { "balance": "1606938044258990275541962092341162602522202993782792835301376" }
    }
}

```
Run Parity:
```
parity --chain ./wasm-dev-chain.json --jsonrpc-apis=all
```
Open a separate terminal window, cd to `step-4` and build contract:
```bash
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target pwasm_tutorial_contract
```
It should produce 2 files:
- a compiled Wasm binary `./target/pwasm_tutorial_contract.wasm`
- an ABI file: `./target/json/TokenContract.json`

Now you can use Web.js to connect to the Parity node and deploy Wasm `pwasm_tutorial_contract.wasm`:

```javascript
    var Web3 = require("web3");
    var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
    web3.eth.defaultAccount = "0x00a329c0648769a73afac7f9381e08fb43dbea72";

    var abi = JSON.parse(fs.readFileSync("./target/json/TokenContract.json")); // read JSON ABI
    var codeHex = '0x' + fs.readFileSync("./target/pwasm_tutorial_contract.wasm").toString('hex'); // convert Wasm binary to hex format

    var TokenContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount });
    // Will create TokenContract with `totalSupply` = 10000000 and print a result
    TokenContract.deploy({data: codeHex, arguments: [10000000]}).send({from: web3.eth.defaultAccount}).then((a) => console.log(a);
```

## Testing
TODO: describe testing
