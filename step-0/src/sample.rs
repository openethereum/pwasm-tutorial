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
