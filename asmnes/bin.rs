use asmnes::*;

const SIMPLE: &str = include_str!("../asmexamples/simple.asm");

fn main() -> Result<(), AsmnesError> {
    println!("hello!");
    //println!("{:?}", simple_assemble("hello").unwrap());
    let lines = lex(SIMPLE)?;
    println!("Lex output:");
    println!("{:?}", lines);
    Ok(())
}

