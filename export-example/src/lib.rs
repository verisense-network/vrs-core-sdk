use vrs_core_sdk::{
    codec::{Decode, Encode},
    export, get, init, post, storage,
};
mod quote;
// use quote::T;

#[export]
type G<T> = Result<T, String>;

#[derive(Debug, Decode, Encode)]
#[export]
pub struct E {
    pub a: Vec<u32>,
    pub b: i32,
    pub c: u32,
}

#[derive(Debug, Decode, Encode)]
#[export]
pub enum MyCustomEnum {
    VariantA,
    VariantB(u32),
    VariantC { id: u64, name: String },
}
#[derive(Debug, Decode, Encode)]
#[export]
pub struct D {
    pub b: i32,
}
#[derive(Debug, Decode, Encode)]
#[export]
pub struct B {
    pub c: C,
}

#[derive(Debug, Decode, Encode)]
#[export]
pub struct C {
    pub d: Vec<u32>,
    pub e: (String, String),
}

#[init]
pub fn init() {}

#[post]
pub fn tv(_a: G<u32>) -> G<()> {
    Ok(())
}
#[post]
pub fn a(_a: A) -> Result<(), String> {
    Ok(())
}

#[post]
pub fn bbb(_a: (A, String)) -> Result<(), String> {
    Ok(())
}

#[post]
pub fn cc(a: String, b: String) -> Result<String, String> {
    // cross char in a and char in b to  gernerate c
    if a.len() != b.len() {
        return Err("a and b should have the same length".to_string());
    }
    let mut c = String::new();
    let mut a_iter = a.chars();
    let mut b_iter = b.chars();
    loop {
        match (a_iter.next(), b_iter.next()) {
            (Some(a), Some(b)) => {
                c.push(a);
                c.push(b);
            }
            _ => {
                break;
            }
        }
    }
    Ok(c)
}

#[post]
pub fn use_codec(d: D) -> Result<E, String> {
    Ok(E {
        a: vec![d.b as u32],
        b: d.b,
        c: 0,
    })
}

#[get]
pub fn should_not_call_put() -> Result<(), String> {
    let vec = vec![0u8; 65536 * 4];
    storage::put(b"aaaaaaaaaaaaaaaaaaaaa", &vec).map_err(|e| e.to_string())
}

#[post]
pub fn should_call_put() -> Result<(), String> {
    let vec = vec![0u8; 65536 * 4];
    storage::put(b"bbbbbbbbbbbbbbbbbbbbb", &vec).map_err(|e| e.to_string())
}

#[derive(Debug, Decode, Encode)]
#[export]
pub struct A {
    pub b: B,
    pub tuple_field: (u32, String),
    pub array_field: [u8; 10],
    pub slice_field: Vec<u64>,
    pub ggg: quote::T,
}
