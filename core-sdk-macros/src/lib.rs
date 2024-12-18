use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, Index, ItemFn, ReturnType, Visibility};

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, func: TokenStream) -> TokenStream {
    expand(func, "post")
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, func: TokenStream) -> TokenStream {
    expand(func, "get")
}

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_init");
    expand_init(func, func_name)
}

#[proc_macro_attribute]
pub fn timer(_attr: TokenStream, func: TokenStream) -> TokenStream {
    expand(func, "timer")
}

#[proc_macro_attribute]
pub fn callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_http_callback");
    expand_no_return(func, func_name)
}

fn expand_init(func: ItemFn, entry_name: Ident) -> TokenStream {
    let func_block = &func.block;
    let func_decl = &func.sig;
    let origin_name = &func_decl.ident;
    let func_generics = &func_decl.generics;
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;

    // Ensure the function has no generics
    if !func_generics.params.is_empty() {
        panic!("init function should not have generics");
    }

    // Ensure the function has no parameters
    if !func_inputs.is_empty() {
        panic!("init function should not have any parameters");
    }

    // Ensure the function has a default return type
    if !matches!(func_output, ReturnType::Default) {
        panic!("init function should have a default return type");
    }

    // Generate the code for a function with no parameters
    let expanded = quote! {
        #[no_mangle]
        pub fn #entry_name() {
            // Call the original function directly as it has no parameters
            fn #origin_name() #func_block
            #origin_name();
        }
    };

    expanded.into()
}

fn expand_no_return(func: ItemFn, entry_name: Ident) -> TokenStream {
    let func_block = &func.block;
    let func_decl = &func.sig;
    let origin_name = &func_decl.ident;
    let func_generics = &func_decl.generics;
    let func_inputs = &func_decl.inputs;
    let func_output = &func_decl.output;
    if !func_generics.params.is_empty() {
        panic!("init function should not have generics");
    }
    if !matches!(func_output, ReturnType::Default) {
        panic!("init function should have default return type");
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
    let expanded = quote! {
        #[no_mangle]
        pub fn #entry_name(__ptr: *const u8, __len: usize) {
            let mut v = unsafe { std::slice::from_raw_parts(__ptr, __len) };
            let decoded = <(#(#tys,)*) as ::vrs_core_sdk::codec::Decode>::decode(&mut v).unwrap();
            fn #origin_name((#(#arg_names,)*): (#(#tys,)*)) #func_block
            #origin_name(decoded);
        }
    };
    expanded.into()
}

fn expand(item: TokenStream, rename_prefix: &str) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let decoded_fn_name = format_ident!("__nucleus_{}_{}", rename_prefix, fn_name);
    input_fn.vis = Visibility::Inherited;
    let return_type = &input_fn.sig.output;
    let args = &input_fn.sig.inputs;
    let output_type = match return_type {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };
    let param_types: Vec<_> = args
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(&*pat_type.ty)
            } else {
                None
            }
        })
        .collect();
    let tuple_type = quote! { (#(#param_types,)*) };
    let function_call = {
        let arg_indices: Vec<Index> = (0..param_types.len()).map(|i| Index::from(i)).collect();
        quote! {
            let mut args_bytes = unsafe { std::slice::from_raw_parts(__ptr, __len) };
            let args: #tuple_type = match ::vrs_core_sdk::codec::Decode::decode(&mut args_bytes) {
                Ok(tuple) => tuple,
                Err(_) => return encode_error("Failed to decode arguments tuple".to_string()),
            };
            let result = #fn_name(#(args.#arg_indices,)*);
        }
    };
    let type_name = quote::format_ident!(
        "_NUCLEUS_{}_PARAMS_TYPE_{}",
        rename_prefix.to_uppercase(),
        fn_name
    );
    let type_def = quote! {
        #[allow(non_camel_case_types)]
        type #type_name = #tuple_type;
    };
    let expanded = quote! {
        #type_def
        #input_fn
        #[no_mangle]
        pub fn #decoded_fn_name(__ptr: *const u8, __len: usize) -> *const u8 {
            fn encode_result<T: ::vrs_core_sdk::codec::Encode>(result: T) -> *const u8 {
                let encoded = <T as ::vrs_core_sdk::codec::Encode>::encode(&result);
                let len = encoded.len() as u32;
                let mut output = Vec::with_capacity(4 + len as usize);
                output.extend_from_slice(&len.to_ne_bytes());
                output.extend_from_slice(&encoded);
                let ptr = output.as_ptr();
                std::mem::forget(output);
                ptr
            }
            fn encode_error(error: String) -> *const u8 {
                encode_result(Err::<Vec<u8>, String>(error))
            }
            #function_call
            let encoded = <#output_type as ::vrs_core_sdk::codec::Encode>::encode(&result);
            let wrapped_result: Result<Vec<u8>, String> = Ok(encoded);
            encode_result(wrapped_result)
        }
    };
    TokenStream::from(expanded)
}
