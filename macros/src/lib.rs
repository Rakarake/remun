extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

// Haha funny
#[proc_macro]
pub fn funny_number(_item: TokenStream) -> TokenStream {
    let tokens = quote! {
        fn big_chungus(a: i32) -> i32 { a + 10 }
        fn small_chungus(b: i8) -> i8 { b + 20 }
    };
    tokens.into()
    //"{ 21 }".parse().unwrap()
}

// Produces the following:
// 1. An enum "Opcode" (enum Opcode {LDA, STA ...})
// 2. A function "parse_operand"
// fn parse_operand(opcode: Opcode) -> Box<dyn Fn(&str) -> IResult<&str, Operand, ErrorTree<&str>>>
// 3. A function "parse_opcode" 
// fn parse_opcode(i: &str) -> IResult<&str, Opcode, ErrorTree<&str>>
#[proc_macro]
pub fn from_table(_item: TokenStream) -> TokenStream {
    "32".parse().unwrap()
}

