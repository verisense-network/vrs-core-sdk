## Verisense Core SDK

![docs.rs](https://img.shields.io/docsrs/vrs-core-sdk)

The core SDK for developing Verisense Nucleus. For more details, see the [Documentation](https://docs.verisense.network).

## Quick Start


``` toml
[package]
name = "hello_avs"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
parity-scale-codec = { version = "3.6", features = ["derive"] }
scale-info = { features = ["derive", "serde"], version = "2.11", default-features = false }
vrs-core-sdk = { version = "0.2" }
```

```rust
use vrs_core_sdk::codec::{Decode, Encode};
use vrs_core_sdk::scale_info::TypeInfo;
use vrs_core_sdk::{get, nucleus, post, storage};

// This is a simple example of a nucleus that manages users.
// It provides two endpoints: one to add a user and another to retrieve a user by ID.
#[nucleus]
pub mod nucleus {
    use super::*;

    #[derive(Debug, Decode, Encode, TypeInfo)]
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
}
```


## Interacting with Nucleus

Since version 0.2, the ABI will be automatically generated if the `#[get]` and `#[post]` functions are within a mod with `#[nucleus]` annotated.

You could request the `abi` method from an RPC node.

``` bash
curl localhost:9955 -H'content-type:application/json' -d'{"jsonrpc":"2.0","id":1,"method":"nucleus_abi","params":["kGjdLfHwt3NFrDW6SsCP6B194oA2xCY95CG5LZd5AyC1PM3Hf"]}'
```

or

``` bash
curl localhost:9955/kGjdLfHwt3NFrDW6SsCP6B194oA2xCY95CG5LZd5AyC1PM3Hf -H'content-type:application/json' -d'{"jsonrpc":"2.0","id":1,"method":"abi","params":[]}'
```
