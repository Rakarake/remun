use asmnes::*;

const SIMPLE: &str = include_str!("../asmexamples/simple.asm");

fn main() -> Result<(), AsmnesError> {
    println!("hello!");
    //println!("{:?}", simple_assemble("hello").unwrap());
    let lex_output = lex(SIMPLE)?;
    let parse_output = parse(lex_output.clone())?;
    println!("Lex output: {:?}", lex_output);
    println!("Parse output: {:?}", parse_output);
    Ok(())
}

