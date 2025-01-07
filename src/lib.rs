use proc_macro::TokenStream;

/// `#[lingo_star::ignore]` has absolutely no effect but it is recognized by lingo* tooling
#[proc_macro_attribute]
pub fn ignore(attr: TokenStream, item: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as syn::parse::Nothing);

    item
}
