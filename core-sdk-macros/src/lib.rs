use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_{}_{}", "post", &func.sig.ident);
    expand(func, func_name)
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_{}_{}", "get", &func.sig.ident);
    expand(func, func_name)
}

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_init");
    expand(func, func_name)
}

#[proc_macro_attribute]
pub fn timer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_{}_{}", "timer", &func.sig.ident);
    expand(func, func_name)
}

#[proc_macro_attribute]
pub fn callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_http_callback");
    expand(func, func_name)
}

fn expand(func: ItemFn, entry_name: Ident) -> TokenStream {
    let func_block = &func.block;
    let func_decl = &func.sig;
    let origin_name = &func_decl.ident;
    let func_generics = &func_decl.generics;
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;
    if !func_generics.params.is_empty() {
        panic!("function should not have generics");
    }
    let tys: Vec<_> = func_inputs
        .iter()
        .map(|i| match i {
            FnArg::Typed(ref val) => val.ty.clone(),
            _ => unreachable!(),
        })
        .collect();
    let arg_names: Vec<_> = func_inputs
        .iter()
        .map(|i| match i {
            FnArg::Typed(ref val) => val.pat.clone(),
            _ => unreachable!(),
        })
        .collect();
    let out_ty = match func_output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };
    let expanded = quote! {
        // declare the wrapper function: `fn __nucleus_XX(__ptr: *const u8, __len: usize)`
        #[no_mangle]
        pub fn #entry_name(__ptr: *const u8, __len: usize) -> *const u8 {
            // rewrite the original function `fn(x: X, y: Y)` to `fn((x, y, z): (X, Y, Z))`
            fn #origin_name((#(#arg_names,)*): (#(#tys,)*)) #func_output #func_block
            // the VM has passed the raw parameters, now decode them within VM
            let mut v = unsafe { std::slice::from_raw_parts(__ptr, __len) };
            let decoding_result = <(#(#tys,)*) as ::vrs_core_sdk::codec::Decode>::decode(&mut v);
            let result: Option<Vec<u8>> = match decoding_result {
                Ok(decoded) => {
                    let ret = #origin_name(decoded);
                    let encoded = <#out_ty as ::vrs_core_sdk::codec::Encode>::encode(&ret);
                    Option::<Vec<u8>>::Some(encoded)
                }
                Err(_) => None::<Vec<u8>>,
            };
            let encoded = <Option<Vec<u8>> as ::vrs_core_sdk::codec::Encode>::encode(&result);
            let len = encoded.len() as u32;
            let mut output = Vec::with_capacity(4 + len as usize);
            output.extend_from_slice(&len.to_ne_bytes());
            output.extend_from_slice(&encoded);
            let ptr = output.as_ptr();
            std::mem::forget(output);
            ptr
        }
    };
    expanded.into()
}
