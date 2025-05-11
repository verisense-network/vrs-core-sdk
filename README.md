# Verisense Core SDK

> The core sdk for developing [Verisense](https://verisense.network) nucleus. Check [Docs](https://docs.verisense.network) for more details.

## Quick start

Store/retrieve a `User` on/from Verisense nucleus.

``` rust
use parity_scale_codec::{Decode, Encode};
use vrs_core_sdk::{get, post, storage};

#[derive(Debug, Decode, Encode)]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[post]
pub fn add_user(user: User) -> Result<u64, String> {
    let max_id_key = [&b"user:"[..], &u64::MAX.to_be_bytes()[..]].concat();
    let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
        .map_err(|e| e.to_string())?
    {
        Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
        None => 1u64,
    };
    let key = [&b"user:"[..], &max_id.to_be_bytes()[..]].concat();
    storage::put(&key, user.encode()).map_err(|e| e.to_string())?;
    Ok(max_id)
}

#[get]
pub fn get_user(id: u64) -> Result<Option<User>, String> {
    let key = [&b"user:"[..], &id.to_be_bytes()[..]].concat();
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let user = r.map(|d| User::decode(&mut &d[..]).unwrap());
    Ok(user)
}
```
