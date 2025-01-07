use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input, ItemFn};

/// `#[lingo_star::ignore]` has absolutely no effect but it is recognized by lingo* tooling
#[proc_macro_attribute]
pub fn ignore(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_macro_input!(attr as syn::parse::Nothing);

    item
}
