mod iso639;

fn to_camel_case(name: &str) -> String {
    use heck::ToUpperCamelCase;

    let cleaned = name
        .split("(")
        .next()
        .unwrap_or(name)
        .replace(',', "")
        .replace('-', " ")
        .replace('_', " ");

    cleaned.trim().to_upper_camel_case()
}

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::iso639::read_csv;

#[proc_macro]
pub fn generate_language(_: TokenStream) -> TokenStream {
    let mut tokens = proc_macro2::TokenStream::new();

    let csv = read_csv().unwrap();
    let mut from_tos: HashMap<String, Vec<(Ident, Option<String>)>> = HashMap::new();
    let variant_idents = csv.into_iter().map(|map| {
        let name = map.get("name").as_ref().unwrap().as_ref().unwrap();
        let enum_name = format_ident!("{}", to_camel_case(name));
        for (key, item) in map {
            from_tos
                .entry(key.to_owned())
                .or_default()
                .push((enum_name.clone(), item));
        }
        enum_name
    });
    tokens.extend(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Language {
            #(#variant_idents),*
        }
    });

    for (key, variants) in from_tos {
        let fn_from = format_ident!("from_{}", key.replace("/", "").replace("-", "_"));
        let fn_to = format_ident!("to_{}", key.replace("/", "").replace("-", "_"));

        let from_arms = variants
            .iter()
            .filter_map(|v| v.1.clone().map(|vv| (v.0.clone(), vv)))
            .map(|(variant, val)| {
                quote! { #val => Some(Language::#variant), }
            });

        let to_arms = variants
            .iter()
            .filter_map(|v| v.1.clone().map(|vv| (v.0.clone(), vv)))
            .map(|(variant, val)| {
                quote! { Language::#variant => Some(#val), }
            });

        let impl_block = quote! {
            impl Language {
                pub fn #fn_from(s: &str) -> Option<Self> {
                    match s {
                        #(#from_arms)*
                        _ => None,
                    }
                }

                pub fn #fn_to(&self) -> Option<&'static str> {
                    match self {
                        #(#to_arms)*
                        _ => None,
                    }
                }
            }
        };
        tokens.extend(impl_block);
    }

    tokens.into()
}
