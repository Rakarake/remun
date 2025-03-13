#[cfg(test)]
mod test_general {
    use asmnes::AsmnesError;
    use asmnes::AsmnesOutput;
    use remun::State;
    /// Helper to run a simple program.
    fn run_program(n_instructions: u64, program: &str) -> Result<State, AsmnesError> {
        let AsmnesOutput { banks, labels: _ } = asmnes::assemble(program)?;
        let mut state = State::new_nrom128(banks[0].data.clone(), banks[1].data.clone());
        state.run_instructions(n_instructions);
        Ok(state)
    }

    #[test]
    fn test_transfer_registers_by_label() -> Result<(), AsmnesError> {
        //state.print_state();
        //assert!(state.x == 0x02);
        //assert!(state.y == 0x04);
        Ok(())
    }
}
