//! The core sdk for developing nucleus on Verisense.
//! NOTE: This crate is currently under heavy development and is not stable yet. We release it just for testing and collecting feedback.
//!
//! # Examples
//!
//! ```
//! use parity_scale_codec::{Decode, Encode};
//! use vrs_core_sdk::{get, post, storage};
//!
//! #[derive(Debug, Decode, Encode)]
//! pub struct User {
//!     pub id: u64,
//!     pub name: String,
//! }
//!
//! #[post]
//! pub fn add_user(user: User) -> Result<u64, String> {
//!     let max_id_key = [&b"user:"[..], &u64::MAX.to_be_bytes()[..]].concat();
//!     let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
//!         .map_err(|e| e.to_string())?
//!     {
//!         Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
//!         None => 1u64,
//!     };
//!     let key = [&b"user:"[..], &max_id.to_be_bytes()[..]].concat();
//!     storage::put(&key, user.encode()).map_err(|e| e.to_string())?;
//!     Ok(max_id)
//! }
//!
//! #[get]
//! pub fn get_user(id: u64) -> Result<Option<User>, String> {
//!     let key = [&b"user:"[..], &id.to_be_bytes()[..]].concat();
//!     let r = storage::get(&key).map_err(|e| e.to_string())?;
//!     let user = r.map(|d| User::decode(&mut &d[..]).unwrap());
//!     Ok(user)
//! }
//! ```

pub mod abi;
pub mod error;
pub mod http;
pub mod io;
pub mod storage;
pub mod timer;
pub mod tss;

pub use codec;
pub use io::{_eprint, _print, nucleus_id};
pub use lazy_static;
pub use scale_info;
pub use sp_core::crypto::AccountId32 as AccountId;
pub use vrs_core_macros::*;

/// the buffer used for transfering data from host to wasm
/// this should be equal to a page size
pub const BUFFER_LEN: usize = 64 * 1024;

/// if host function returns this value, it means there is no more data to read
pub const NO_MORE_DATA: i32 = 0;

/// result of host function, T should be `codec::Codec`
pub type CallResult<T> = Result<T, error::RuntimeError>;

/// the id of the nucleus, same as AccountId32
pub type NucleusId = AccountId;

#[inline]
pub(crate) fn allocate_buffer() -> Vec<u8> {
    vec![0u8; BUFFER_LEN]
}
