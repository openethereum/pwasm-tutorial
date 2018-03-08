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
