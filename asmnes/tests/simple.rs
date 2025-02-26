#[cfg(test)]
mod test_simple {
    use asmnes::*;
    const SIMPLE: &str = include_str!("../../asmexamples/simple.asm");
    #[test]
    fn test_lexer() -> Result<(), AsmnesError> {
        let lines = lex(SIMPLE)?;
        println!("Lex output:");
        println!("{:?}", lines);
        Ok(())
    }
}
