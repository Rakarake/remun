use asmnes::*;

const SIMPLE: &str = include_str!("../asmexamples/simple.asm");

fn main() -> Result<(), AsmnesError> {
    println!("hello!");
    //println!("{:?}", simple_assemble("hello").unwrap());
    let lex_output = lex(SIMPLE)?;
    let parse_output = parse(lex_output.clone())?;
    //let logical_output = logical_assemble(program)
    println!("Lex output: {:?}", lex_output);
    println!("Parse output: {:?}", parse_output);
    //println!("Logical output: {:?}", parse_output);
    Ok(())
}

