//! Storage module provides a set of functions to interact with the kv storage.
//! Any data stored in the kv storage will be synchronized across all nodes in the subnet.
//! The nucleus will automatically update the state root after each modification and then
//! submit to Verisense chain.
//!
//! The `put` and `del` can only be called in the post functions. Otherwise, it will case panic.

use crate::{error::RuntimeError, CallResult};
use codec::{Decode, Encode};
use vrs_core_macros::export;
#[repr(u8)]
#[derive(Encode, Decode)]
#[export]
pub enum CryptoType {
    P256 = 0,
    Ed25519 = 1,
    Secp256k1 = 2,
    Secp256k1Tr = 3,
    Ed448 = 4,
    Ristretto255 = 5,
    EcdsaSecp256k1 = 6,
}

impl From<CryptoType> for u8 {
    fn from(value: CryptoType) -> Self {
        value as u8
    }
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn tss_get_public_key_host_fn(
        crypto_type: u8,
        tweak_ptr: *const u8,
        tweak_len: i32,
        return_ptr: *mut u8,
    ) -> i32;
    fn tss_sign_host_fn(
        crypto_type: u8,
        tweak_ptr: *const u8,
        tweak_len: i32,
        message_ptr: *const u8,
        message_len: i32,
        return_ptr: *mut u8,
    ) -> i32;
}

/// get the public key of the given crypto type with the tweak
pub fn tss_get_public_key(crypto_type: CryptoType, tweak: impl AsRef<[u8]>) -> CallResult<Vec<u8>> {
    let tweak = tweak.as_ref();
    let mut buf = crate::allocate_buffer();
    let status = unsafe {
        tss_get_public_key_host_fn(
            crypto_type.into(),
            tweak.as_ptr(),
            tweak.len() as i32,
            buf.as_mut_ptr(),
        )
    };
    assert!(status == crate::NO_MORE_DATA);
    CallResult::<Vec<u8>>::decode(&mut &buf[..])
        .map_err(|_| RuntimeError::DecodeReturnValueError)?
}

/// sign the message with the given crypto type and tweak
pub fn tss_sign(
    crypto_type: CryptoType,
    tweak: impl AsRef<[u8]>,
    message: impl AsRef<[u8]>,
) -> CallResult<Vec<u8>> {
    let tweak = tweak.as_ref();
    let message = message.as_ref();
    let mut buf = crate::allocate_buffer();
    let status = unsafe {
        tss_sign_host_fn(
            crypto_type.into(),
            tweak.as_ptr(),
            tweak.len() as i32,
            message.as_ptr(),
            message.len() as i32,
            buf.as_mut_ptr(),
        )
    };
    assert!(status == crate::NO_MORE_DATA);
    CallResult::<Vec<u8>>::decode(&mut &buf[..])
        .map_err(|_| RuntimeError::DecodeReturnValueError)?
}
