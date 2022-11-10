use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{__private::ToTokens, parse::Parse, Token};

mod custom_kw {
    syn::custom_keyword!(irq);
    syn::custom_keyword!(except);
}

enum Kind {
    IRQ(custom_kw::irq),
    Except(custom_kw::except),
}

impl Parse for Kind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(custom_kw::irq) {
            Ok(Self::IRQ(input.parse()?))
        } else if input.peek(custom_kw::except) {
            Ok(Self::Except(input.parse()?))
        } else {
            Err(input.error("excpected either 'irq' or 'except' as the first tokens."))
        }
    }
}

#[allow(unused)]
struct Def {
    kind: Kind,
    ident: Ident,
    colon0: Token![:],
    id: syn::LitInt,
    arrow: Token![=>],
    handler_name: Ident,
    colon1: Token![:],
    fn_kw: Token![fn],
    func: syn::Block,
}

impl Parse for Def {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind = input.parse()?;
        let ident = input.parse()?;
        let colon0 = input.parse()?;
        let id = input.parse()?;
        let arrow = input.parse()?;
        let handler_name = input.parse()?;
        let colon1 = input.parse()?;
        let fn_kw = input.parse()?;
        let func = input.parse()?;
        Ok(Self {
            kind,
            ident,
            colon0,
            id,
            arrow,
            handler_name,
            colon1,
            fn_kw,
            func,
        })
    }
}

pub fn proc(ts: TokenStream) -> TokenStream {
    impl ToTokens for Def {
        fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
            let Self {
                kind,
                ident,
                id,
                handler_name,
                func,
                ..
            } = self;
            let handler_name = Ident::new(
                &format!("{}{}", handler_name, id.base10_digits()),
                handler_name.span(),
            );
            let kind_dif = match kind {
                Kind::IRQ(_) => 8,
                Kind::Except(_) => 16,
            };
            let asm = format!(
                "
                    .align 8
                    .global {ident}
                    {ident}:
                        push {id}
                        push rax
                        push rbx
                        push rcx
                        push rdx
                        push rsi
                        push rdi
                        push r8
                        push r9
                        push r10
                        push r11
                        push r12
                        push r13
                        push r14
                        push r15
                        push rbp
                        cld
                        mov rdi, rsp
                        .extern {handler_name}
                        call {handler_name}
                        mov rsp, rax
                        pop rbp
                        pop r15
                        pop r14
                        pop r13
                        pop r12
                        pop r11
                        pop r10
                        pop r9
                        pop r8
                        pop rdi
                        pop rsi
                        pop rdx
                        pop rcx
                        pop rbx
                        pop rax
                        add rsp, {kind_dif}
                        iretq
                "
            );
            tokens.extend(quote! {
                ::core::arch::global_asm!(#asm);
                #[doc(hidden)]
                #[allow(unused)]
                #[no_mangle]
                unsafe extern "C" fn #handler_name (
                    stack_frame: *mut StackFrame,
                ) -> *mut StackFrame {
                    { #func };
                    pic::end_of_interrupt();
                    stack_frame
                }
                extern "C" {
                    fn #ident();
                }
            });
        }
    }

    let def: Def = syn::parse(ts).expect("unable to create x86_64 isr");
    quote!(#def).into()
}
