use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, visit_mut::VisitMut, Attribute, FnArg, Ident, ItemFn, ItemMod,
    PatType, ReturnType, Type,
};

#[derive(Clone)]
struct ApiEntry {
    name: String,
    method: String,
    param_types: Vec<Box<Type>>,
    return_type: Box<Type>,
}

struct ApiVisitor {
    entries: Vec<ApiEntry>,
}

fn find_entry(attrs: &[Attribute]) -> Option<&str> {
    for attr in attrs {
        if attr.path().is_ident("get") {
            return Some("get");
        } else if attr.path().is_ident("post") {
            return Some("post");
        }
    }
    None
}

impl VisitMut for ApiVisitor {
    fn visit_item_fn_mut(&mut self, item: &mut syn::ItemFn) {
        if let Some(method) = find_entry(&item.attrs) {
            let name = item.sig.ident.to_string();
            let param_types: Vec<_> = item
                .sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(PatType { ty, .. }) => Some((*ty).clone()),
                    _ => None,
                })
                .collect();
            let return_type = match &item.sig.output {
                ReturnType::Default => parse_quote!(()),
                ReturnType::Type(_, ty) => (*ty).clone(),
            };
            self.entries.push(ApiEntry {
                name,
                method: method.to_string(),
                param_types,
                return_type,
            });
        }
        syn::visit_mut::visit_item_fn_mut(self, item);
    }
}

#[proc_macro_attribute]
pub fn nucleus(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_mod = parse_macro_input!(item as ItemMod);
    let mut visitor = ApiVisitor {
        entries: Vec::new(),
    };
    visitor.visit_item_mod_mut(&mut input_mod);
    let entries: Vec<_> = visitor
        .entries
        .iter()
        .map(|entry| {
            let name = &entry.name;
            let method = &entry.method;
            let param_types = entry
                .param_types
                .iter()
                .map(|ty| ty.clone())
                .collect::<Vec<_>>();
            let return_type = &entry.return_type;
            quote! {
                registry.register_api(
                    #name.to_string(),
                    #method.to_string(),
                    vec![#(::vrs_core_sdk::scale_info::meta_type::<#param_types>(),)*],
                    ::vrs_core_sdk::scale_info::meta_type::<#return_type>(),
                );
            }
        })
        .collect::<Vec<_>>();
    if let Some((_, ref mut items)) = input_mod.content {
        items.push(parse_quote! {
            vrs_core_sdk::lazy_static::lazy_static! {
                static ref TYPES: ::vrs_core_sdk::abi::ApiRegistry = {
                    let mut registry = ::vrs_core_sdk::abi::ApiRegistry::new();
                    #(#entries)*
                    registry
                };
            }
        });
        items.push(parse_quote! {
            #[no_mangle]
            pub fn __nucleus_abi() -> *const u8 {
                let abi = TYPES.dump_abi();
                let encoded = <::vrs_core_sdk::abi::JsonAbi as ::vrs_core_sdk::codec::Encode>::encode(&abi);
                let dummy_encoded = Some(encoded);
                let encoded = <Option<Vec<u8>> as ::vrs_core_sdk::codec::Encode>::encode(&dummy_encoded);
                let len = encoded.len() as u32;
                let mut output = Vec::with_capacity(4 + len as usize);
                output.extend_from_slice(&len.to_ne_bytes());
                output.extend_from_slice(&encoded);
                let ptr = output.as_ptr();
                std::mem::forget(output);
                ptr
            }
        });
    }
    quote! {
        #input_mod
    }
    .into()
}

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
