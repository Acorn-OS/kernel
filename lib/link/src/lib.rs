use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::{collections::HashSet, sync::Mutex};
use syn::{parenthesized, parse::Parse, punctuated::Punctuated, FnArg, ReturnType, Token};

lazy_static! {
    static ref NAMES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

struct Unpacked {
    fn_tkn: Token![fn],
    ident: Ident,
    #[allow(unused)]
    paren: syn::token::Paren,
    args: Punctuated<FnArg, Token![,]>,
    tail: ReturnType,
}

impl Parse for Unpacked {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fn_tkn = input.parse()?;
        let ident = input.parse()?;
        let content;
        let paren = parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;
        let tail = input.parse()?;
        Ok(Self {
            fn_tkn: fn_tkn,
            ident: ident,
            paren: paren,
            args: args,
            tail,
        })
    }
}

#[proc_macro]
pub fn links(ts: TokenStream) -> TokenStream {
    let unpacked = syn::parse::<Unpacked>(ts).expect("parsing failed.");
    parse(unpacked)
}

fn parse(unpacked: Unpacked) -> TokenStream {
    let Unpacked {
        fn_tkn,
        ident,
        paren: _,
        args,
        tail,
    } = unpacked;
    let ident_str = ident.to_string();
    let link_name = format!("__link__{ident_str}__",);

    if NAMES.lock().unwrap().contains(&link_name) {
        panic!("multiple definitions of '{ident_str}'")
    } else {
        NAMES.lock().unwrap().insert(link_name.clone());
    }

    let macro_ident_str = format!("links_{ident_str}");
    let macro_ident = Ident::new(&macro_ident_str, Span::call_site());
    let arg_idents = args.clone();
    let arg_idents = arg_idents.iter().map(|e| match e {
        FnArg::Receiver(_) => panic!("does not accept 'self' as an argument here."),
        FnArg::Typed(pat) => match pat.pat.as_ref() {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => panic!("expected ident in pattern"),
        },
    });
    quote! {
        extern "Rust" {
            #[link_name = #link_name]
            pub #fn_tkn #ident(#args) #tail;
        }
        #[macro_export]
        macro_rules! #macro_ident {
            ($ident:ident) => {
                #[doc(hidden)]
                #[export_name = #link_name]
                pub extern "Rust" fn #ident(#args) #tail{
                    $ident(#(#arg_idents),*)
                }
            };
        }
    }
    .into()
}
