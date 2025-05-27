use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, ItemType, ReturnType};

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
use std::env;
use std::path::PathBuf;
fn get_export_file() -> PathBuf {
    // 1. Get the name of the crate currently being compiled
    let pkg_name = env::var("CARGO_PKG_NAME")
        .expect("CARGO_PKG_NAME environment variable not set; ensure you are compiling within a Cargo environment.");
    // 2. Get the output directory (use env var if set, otherwise default to target/json_exports)
    let export_dir_str =
        env::var("EXPORT_JSON_DIR").unwrap_or_else(|_| "target/json_exports".to_string());

    let export_dir = PathBuf::from(export_dir_str);

    // 3. Ensure the output directory exists (Note: potential race in parallel builds, but create_dir_all is usually okay)
    std::fs::create_dir_all(&export_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create export directory: {:?}. Error: {}",
            export_dir, e
        )
    });

    // 4. Build and return the unique file path, e.g., target/json_exports/my_crate_name.json
    export_dir.join(format!("__export_json_crate_{}.json", pkg_name))
}
use serde::Serialize;
use std::{fs::OpenOptions, io::Write};

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

/// JSON structure definition for Type Aliases
#[derive(Serialize)]
struct ExportTypeAlias {
    r#type: &'static str, // Will be "type_alias" or "type"
    name: String,
    generics: Vec<String>,
    target: TypePath,
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
pub fn export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let ast: syn::Item = parse_macro_input!(input_clone);

    match &ast {
        syn::Item::Struct(s) => export_struct(&s),
        syn::Item::Type(alias) => export_type_alias(&alias),
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
fn export_type_alias(t: &ItemType) {
    let generic_params = t
        .generics
        .params
        .iter()
        .filter_map(|gp| match gp {
            syn::GenericParam::Type(tp) => Some(tp.ident.to_string()),
            _ => None,
        })
        .collect();

    let export = ExportTypeAlias {
        r#type: "type_alias",
        name: t.ident.to_string(),
        generics: generic_params,
        target: parse_type(&*t.ty), // t.ty is a Box<Type>, so dereference
    };

    append_to_file(&export);
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

use fs2::FileExt; // Import FileExt for locking
use serde_json::Value;
use std::collections::HashSet; // Import HashSet
use std::io::{Read, Seek, SeekFrom}; // Import IO traits
use std::sync::{LazyLock, Mutex}; // Use Mutex for the map // Ensure Serialize is in scope // Add LazyLock
                                  // Use a Mutex-protected HashSet to track initialized files per build run.
static INIT_MAP: LazyLock<Mutex<HashSet<PathBuf>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

fn append_to_file<T: Serialize>(data: &T) {
    let export_file_path = get_export_file(); // Get the PathBuf

    // --- Use a file lock to ensure atomic RMW operations ---
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&export_file_path)
        .expect("Cannot open or create export file");

    // Acquire an exclusive (write) lock, blocking until available.
    file.lock_exclusive().expect("Cannot acquire file lock");

    // --- Perform initialization and RMW within the lock ---
    let mut needs_init = false;
    {
        // Scope for Mutex guard
        let mut init_set = INIT_MAP.lock().unwrap();
        if !init_set.contains(&export_file_path) {
            // Check if file is empty or doesn't exist. If so, it needs initialization.
            if file.metadata().map(|m| m.len() == 0).unwrap_or(true) {
                needs_init = true;
            }
            // Mark as 'processed' for this build run, even if not empty,
            // to avoid re-checking.
            init_set.insert(export_file_path.clone());
        }
    } // Mutex guard is dropped here

    let mut current_file = &file; // Use a reference to satisfy Write/Read traits

    if needs_init {
        // File is empty or new, write "[]"
        current_file.set_len(0).expect("Cannot truncate file");
        current_file
            .seek(SeekFrom::Start(0))
            .expect("Cannot seek in file");
        current_file
            .write_all(b"[]")
            .expect("Cannot initialize file");
    }

    // Rewind to the beginning to read the whole content
    current_file
        .seek(SeekFrom::Start(0))
        .expect("Cannot seek in file");

    // Read the existing JSON array
    let mut content = String::new();
    current_file
        .read_to_string(&mut content)
        .expect("Cannot read file");

    let mut entries: Vec<Value> = serde_json::from_str(&content).unwrap_or_else(|_err| {
        // If parsing fails (e.g., was empty or corrupted), start fresh.
        // With locking and init, this should ideally not happen unless externally modified.
        if needs_init || content.trim().is_empty() {
            Vec::new()
        } else {
            panic!("Failed to parse existing JSON: {}", content)
        }
    });

    // Serialize new data and append
    let new_entry = serde_json::to_value(data).expect("Serialize error");
    entries.push(new_entry);

    // Serialize back to a JSON string
    let formatted_json = serde_json::to_string_pretty(&entries).expect("JSON serialization error");

    // Truncate the file and write the new content
    current_file.set_len(0).expect("Cannot truncate file");
    current_file
        .seek(SeekFrom::Start(0))
        .expect("Cannot seek in file");
    current_file
        .write_all(formatted_json.as_bytes())
        .expect("Write error");

    // --- Release the file lock ---
    fs2::FileExt::unlock(&file).expect("Cannot release file lock");
}
