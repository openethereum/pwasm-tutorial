#![no_std]
#![feature(wasm_import_memory)]
#![wasm_import_memory]

extern crate pwasm_std;
extern crate pwasm_ethereum;
///
extern crate pwasm_abi;
///
extern crate pwasm_abi_derive;
/// Bigint used for 256-bit arithmetic
extern crate bigint;

mod token {
    use pwasm_ethereum::{storage, ext};
    use pwasm_std::hash::{Address, H256};
    use bigint::U256;
    use pwasm_abi_derive::eth_abi;

    static TOTAL_SUPPLY_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static OWNER_KEY: H256 = H256([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

    // Generates a balance key for some address.
    // Used to map balances with their owners.
    fn balance_key(address: &Address) -> H256 {
        let mut key = H256::from(address);
        key[0] = 1; // just a naiive "namespace";
        key
    }

    // TokenContract is an interface definition of a contract.
    // eth_abi macro parses an interface (trait) definition of a contact and generates
    // two structs: `Endpoint` and `Client`.
    //
    // `Endpoint` is an entry point for contract calls. Endpoint inmlpements the EndpointInterface with 2 methods
    // eth_abi macro generates a table of Method IDs corresponding with every method signature defined in the trait
    // and defines it statically in the generated code.
    // Scroll down at "pub fn call(desc: *mut u8)" to see how.
    // Endpoint instantiates with a struct TokenContractInstance which implements the TokenContract trait definition.
    //
    //
    // `Client` is a struct which is useful for call generation to a deployed contract. For example:
    // ```
    //     let mut client = Client::new(contactAddress);
    //     let balance = client
    //        .value(someValue) // you can attach some value for a call optionally
    //        .balanceOf(someAddress);
    // ```
    // Will generate a Solidity-compatible call for the contract, deployed on `contactAddress`.
    // Then it invokes pwasm_std::ext::call on `contactAddress` and returns the result.
    #[eth_abi(Endpoint, Client)]
    pub trait TokenContract {
        fn constructor(&mut self, _total_supply: U256);
        /// Total amount of tokens
        #[constant]
        fn totalSupply(&mut self) -> U256;
    }

    pub struct TokenContractInstance;

    impl TokenContract for TokenContractInstance {
        fn constructor(&mut self, total_supply: U256) {
            let sender = ext::sender();
            // Set up the total supply for the token
            storage::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
            // Give all tokens to the contract owner
            storage::write(&balance_key(&sender), &total_supply.into());
            // Set the contract owner
            storage::write(&OWNER_KEY, &H256::from(sender).into());
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
    // pwasm_std::parse_args parses the call descriptor into arguments and result pointers
    // Args is an Solidity-compatible abi call: first 4 bytes are the Method ID of keccak hash of function signature
    // followed by sequence of arguments packed into chunks of 32 bytes.
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = token::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = token::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
    endpoint.dispatch_ctor(&args);
}

