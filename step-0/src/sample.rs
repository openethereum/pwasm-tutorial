// Contract doesn't use Rust's standard library
#![no_std]

// `pwasm-std` is the lightweight implementation of a standard library
// It implements common data structures and provides bindings to the runtime
extern crate pwasm_std;

/// Will be described in the next step
#[no_mangle]
pub fn deploy(_desc: *mut u8) {
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
