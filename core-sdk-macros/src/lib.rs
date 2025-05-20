use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, ReturnType};

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_{}_{}", "post", &func.sig.ident);
    export_fn_internal("post", &func);
    expand(func, func_name)
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = format_ident!("__nucleus_{}_{}", "get", &func.sig.ident);
    export_fn_internal("get", &func);
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

fn get_export_file() -> String {
    std::env::var("EXPORT_JSON_FILE").unwrap_or_else(|_| "./exports.json".to_string())
}

use serde::Serialize;
use std::{fs::OpenOptions, io::Write};
use syn::{Data, DeriveInput};

#[derive(Serialize)]
#[serde(tag = "kind")]
enum TypePath {
    Path {
        path: Vec<String>,
        generic_args: Vec<TypePath>,
    },
    Tuple {
        tuple_args: Vec<TypePath>,
    },
    Array {
        elem: Box<TypePath>,
        len: Option<usize>, // None represents slice, Some(len) represents array
    },
    Unsupported,
}
/// JSON structure definition
#[derive(Serialize)]
struct ExportStruct {
    r#type: &'static str,
    name: String,
    fields: Vec<ExportField>,
}

#[derive(Serialize)]
struct ExportField {
    name: String,
    r#type: TypePath,
}

#[derive(Serialize)]
struct ExportFn {
    r#type: &'static str,
    name: String,
    method: String,
    inputs: Vec<ExportField>,
    output: Option<TypePath>,
}

#[proc_macro_attribute]
pub fn export(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let ast: syn::Item = parse_macro_input!(input_clone);

    match &ast {
        syn::Item::Struct(s) => export_struct(&s),
        _ => (),
    }

    // Return original definition unchanged
    input
}
fn parse_type(ty: &syn::Type) -> TypePath {
    match ty {
        syn::Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            let mut path = vec![];
            let mut generic_args = vec![];

            for segment in segments.iter() {
                path.push(segment.ident.to_string());

                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in args.args.iter() {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            generic_args.push(parse_type(inner_ty));
                        }
                    }
                }
            }

            TypePath::Path { path, generic_args }
        }

        syn::Type::Tuple(ty_tuple) => TypePath::Tuple {
            tuple_args: ty_tuple.elems.iter().map(parse_type).collect(),
        },

        syn::Type::Array(ty_array) => {
            let len = match &ty_array.len {
                syn::Expr::Lit(expr_lit) => {
                    if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                        Some(lit_int.base10_parse::<usize>().unwrap_or(0))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            TypePath::Array {
                elem: Box::new(parse_type(&ty_array.elem)),
                len,
            }
        }

        syn::Type::Reference(ty_ref) => parse_type(&ty_ref.elem),

        _ => TypePath::Unsupported,
    }
}
fn export_struct(s: &ItemStruct) {
    let fields = s
        .fields
        .iter()
        .filter_map(|f| {
            f.ident.as_ref().map(|ident| ExportField {
                name: ident.to_string(),
                r#type: parse_type(&f.ty),
            })
        })
        .collect();

    let export = ExportStruct {
        r#type: "struct",
        name: s.ident.to_string(),
        fields,
    };

    append_to_file(&export);
}
fn export_fn_internal(method: &str, f: &ItemFn) {
    let inputs = f
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    Some(ExportField {
                        name: ident.ident.to_string(),
                        r#type: parse_type(&pat_type.ty),
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    let output = match &f.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(parse_type(ty)),
    };

    let export = ExportFn {
        r#type: "fn",
        name: f.sig.ident.to_string(),
        method: method.to_string(),
        inputs,
        output,
    };

    append_to_file(&export);
}

fn type_to_string(ty: &syn::Type) -> String {
    quote!(#ty).to_string().replace(' ', "")
}
use serde_json::Value;
use std::fs::{self};
use std::sync::Once;

static INIT: Once = Once::new();

fn append_to_file<T: Serialize>(data: &T) {
    let export_file = get_export_file();

    INIT.call_once(|| {
        // Initialize as empty array on first write during compilation
        fs::write(&export_file, "[]").expect("Failed to initialize exports.json");
    });

    // Serialize new data
    let new_entry = serde_json::to_value(data).expect("Serialize error");

    // Read existing JSON array
    let mut entries: Vec<Value> = fs::read_to_string(&export_file)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(Vec::new);

    // Append new data
    entries.push(new_entry);

    // Write back to file
    let formatted_json = serde_json::to_string_pretty(&entries).expect("JSON serialization error");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&export_file)
        .expect("Cannot open file");

    file.write_all(formatted_json.as_bytes())
        .expect("Write error");
}
