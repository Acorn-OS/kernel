use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{ItemFn, Signature, Visibility};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[proc_macro_attribute]
pub fn ctor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::Item>(item).expect("expected item");
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = &match item {
        syn::Item::Fn(val) => val,
        _ => panic!("expected fn item"),
    };
    let Signature {
        unsafety,
        abi,
        ident,
        ..
    } = sig;
    assert_valid_vis(vis);
    assert_valid_sig(sig);
    let ctor_ident = Ident::new(
        &format!(
            "__rust_ctor__id_{}__{ident}__",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ),
        Span::call_site(),
    );
    let attrs = if attrs.is_empty() {
        quote!()
    } else {
        quote!(#[#(#attrs)*])
    };
    quote! {
        #attrs
        #vis #unsafety #abi fn #ident () -> () #block

        #[used]
        #[allow(non_upper_case_globals)]
        #[link_section = ".init_array"]
        static #ctor_ident: unsafe extern "C" fn() = {
            unsafe extern "C" fn ctor() {
                #ident ()
            };
            ctor
        };


    }
    .into()
}

fn assert_valid_vis(vis: &Visibility) {
    assert!(
        matches!(vis, Visibility::Inherited),
        "constructor function must be private"
    );
}

fn assert_valid_sig(sig: &Signature) {
    assert!(
        sig.constness.is_none(),
        "constructor function cannot be const"
    );
    assert!(
        sig.asyncness.is_none(),
        "constructor function cannot be async"
    );
    assert!(
        sig.generics.params.is_empty(),
        "constructor function may not be generic"
    );
    assert!(
        sig.inputs.is_empty(),
        "constructor function may not take in any arguments"
    );
    assert!(
        matches!(sig.output, syn::ReturnType::Default),
        "constructor function must return '()'"
    );
}
