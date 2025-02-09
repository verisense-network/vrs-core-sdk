use vrs_core_sdk::{get, tss::tss_get_public_key, tss::tss_sign, tss::CryptoType};

#[get]
pub fn get_public_key(crypto_type: CryptoType, tweak: Vec<u8>) -> Result<Vec<u8>, String> {
    tss_get_public_key(crypto_type, tweak).map_err(|e| e.to_string())
}
#[get]
pub fn sign(crypto_type: CryptoType, tweak: Vec<u8>, message: Vec<u8>) -> Result<Vec<u8>, String> {
    tss_sign(crypto_type, tweak, message).map_err(|e| e.to_string())
}
