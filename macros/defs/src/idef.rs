use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized, parse::Parse, punctuated::Punctuated, Attribute, Block, Expr, FnArg,
    Token, Type,
};

#[allow(dead_code)]
struct StructField {
    ident: Ident,
    colon: Token![:],
    ty: Type,
    eq: Token![=],
    expr: Expr,
}

#[allow(dead_code)]
struct StructDef {
    attrs: Vec<Attribute>,
    static_kw: Token![static],
    ident: Ident,
    eq: Token![=],
    brace: syn::token::Brace,
    fields: Punctuated<StructField, Token![,]>,
}

#[allow(dead_code)]
struct StructFn {
    attrs: Vec<Attribute>,
    pub_kw: Option<Token![pub]>,
    unsafe_kw: Option<Token![unsafe]>,
    fn_kw: Token![fn],
    ident: Ident,
    paran: syn::token::Paren,
    ref_token: Token![&],
    mut_token: Option<Token![mut]>,
    self_kw: Token![self],
    sep: Option<Token![,]>,
    args: Punctuated<FnArg, Token![,]>,
    ret_token: Option<Token![->]>,
    output: Option<Type>,
    block: Block,
}

#[allow(dead_code)]
struct StructImpl {
    impl_kw: Token![impl],
    brace: syn::token::Brace,
    methods: Vec<StructFn>,
}

struct Impl {
    s: StructDef,
    i: Option<StructImpl>,
}

impl Parse for Impl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let s = input.parse::<StructDef>()?;
        let i = if input.peek(Token![impl]) {
            Some(input.parse::<StructImpl>()?)
        } else {
            None
        };
        Ok(Self { s, i })
    }
}

impl Parse for StructField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        let eq = input.parse::<Token![=]>()?;
        let expr = input.parse::<Expr>()?;
        Ok(Self {
            ident,
            colon,
            ty,
            eq,
            expr,
        })
    }
}

impl Parse for StructDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let static_kw = input.parse()?;
        let ident = input.parse()?;
        let eq = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let fields = content.call(Punctuated::parse_terminated)?;
        Ok(Self {
            attrs,
            static_kw,
            ident,
            eq,
            brace,
            fields,
        })
    }
}

impl Parse for StructFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let pub_kw = input.parse()?;
        let unsafe_kw = input.parse()?;
        let fn_kw = input.parse()?;
        let ident = input.parse()?;
        let content;
        let paran = parenthesized!(content in input);
        let ref_token = content.parse()?;
        let mut_token = content.parse()?;
        let self_kw = content.parse()?;
        let sep: Option<_> = content.parse()?;
        let args: Punctuated<_, _> = content.call(Punctuated::parse_terminated)?;
        if !args.is_empty() && !sep.is_some() {
            return Err(input.error("Missing separator after self"));
        }
        let ret_token: Option<_> = input.parse()?;
        let output = if ret_token.is_some() {
            Some(input.parse()?)
        } else {
            None
        };
        let block = input.parse()?;
        Ok(Self {
            attrs,
            pub_kw,
            unsafe_kw,
            fn_kw,
            ident,
            paran,
            ref_token,
            mut_token,
            self_kw,
            sep,
            args,
            ret_token,
            output,
            block,
        })
    }
}

impl Parse for StructImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let impl_kw = input.parse::<Token![impl]>()?;
        let content;
        let brace = braced!(content in input);
        let mut methods = vec![];
        while !content.is_empty() {
            methods.push(content.parse()?);
        }
        Ok(Self {
            impl_kw,
            brace,
            methods,
        })
    }
}

impl ToTokens for Impl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { s, i } = self;
        let imp_tokens = if let Some(i) = i {
            Some(
                i.to_tokens(&s.ident)
                    .expect("Unable to create freestanding function for driver."),
            )
        } else {
            None
        };
        tokens.extend(quote! {
            #s
            #imp_tokens
        });
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, ty, .. } = self;
        tokens.extend(quote!(#ident : #ty));
    }
}

impl ToTokens for StructDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            attrs,
            ident,
            fields,
            ..
        } = self;
        let initializers: Punctuated<proc_macro2::TokenStream, Token![,]> = fields
            .iter()
            .map(|f| {
                let ident = &f.ident;
                let expr = &f.expr;
                quote!(#ident: #expr)
            })
            .collect();
        tokens.extend(quote! {
            #(#attrs)*
            struct #ident { #fields }
            static #ident: ::proc_macro::__private::spin::Mutex<#ident> = ::proc_macro::__private::spin::Mutex::new(#ident{
                #initializers
            });
        });
    }
}

impl ToTokens for StructFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            attrs,
            pub_kw,
            unsafe_kw,
            mut_token,
            ident,
            args,
            output,
            sep,
            ret_token,
            block,
            fn_kw,
            paran: _paran,
            ref_token,
            self_kw,
        } = &self;
        tokens.extend(quote! {
            #(#attrs)*
            #pub_kw #unsafe_kw #fn_kw #ident (#ref_token #mut_token #self_kw #sep #args) #ret_token #output #block
        })
    }
}

impl StructImpl {
    fn to_tokens(&self, ident: &Ident) -> Result<proc_macro2::TokenStream, ()> {
        let Self { methods, .. } = self;
        let mut free_funcs = vec![];
        for f in methods {
            if f.pub_kw.is_some() {
                let f_ident = &f.ident;
                let args = &f.args;
                let attrs = &f.attrs;
                let f_unsafe = &f.unsafe_kw;
                let ret_tk = f.ret_token;
                let ret_ty = &f.output;
                let arg_names = args.iter().map(|arg| match arg {
                    FnArg::Receiver(_) => panic!("multiple self in function"),
                    FnArg::Typed(typed) => match &*typed.pat {
                        syn::Pat::Ident(ident) => ident,
                        _ => panic!("expected ident pattern"),
                    },
                });
                free_funcs.push(quote! {
                    #(#attrs)*
                    pub #f_unsafe fn #f_ident (#args) #ret_tk #ret_ty {
                        (#ident.lock()).#f_ident (#(#arg_names),*)
                    }
                });
            }
        }
        Ok(quote!(
            #[doc(hidden)]
            impl #ident{
                #(#methods)*
            }

            #(#free_funcs)*
        ))
    }
}

pub fn proc(ts: TokenStream) -> TokenStream {
    let i: Impl = syn::parse(ts).expect("Unable to implement driver");
    quote!(#i).into()
}
