mod idef;
mod x86;

use proc_macro::TokenStream;

#[proc_macro]
pub fn idef(ts: TokenStream) -> TokenStream {
    idef::proc(ts)
}

#[proc_macro]
pub fn x86_isr_def(ts: TokenStream) -> TokenStream {
    x86::isr_def::proc(ts)
}
