extern crate proc_macro;
use proc_macro::TokenStream;

// Haha funny
#[proc_macro]
pub fn funny_number(_item: TokenStream) -> TokenStream {
    "{ 21 }".parse().unwrap()
}

