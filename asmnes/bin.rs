use std::error::Error;

use asmnes::*;
use asmnes::lexer::lex;
use asmnes::parser::parse;

const SIMPLE: &str = include_str!("../asmexamples/simple.asm");

fn main() -> Result<(), Box<dyn Error>> {
    println!("hello!");
    //println!("{:?}", simple_assemble("hello").unwrap());
    let lex_output = lex(SIMPLE)?;
    println!("Lex output: {:?}", lex_output);
    let parse_output = parse(lex_output.clone())?;
    println!("Parse output: {:?}", parse_output);
    let logical_output = logical_assemble(&parse_output)?;
    println!("Logical output: {:?}", logical_output.mapper);
    Ok(())
}
