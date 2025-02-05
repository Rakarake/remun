#[cfg(test)]
mod test_general {
    use asmnes::AsmnesError;
    use asmnes::AsmnesOutput;
    use shared::Opcode::*;
    use shared::AddressingMode::*;
    use asmnes::Operand::*;
    use asmnes::INSTR;
    use asmnes::INSTRL;
    use asmnes::Directive;
    use remun::State;

    /// Helper to run a simple program.
    fn run_program(n_instructions: u64, program: &[INSTRL]) -> Result<State, AsmnesError> {
        let AsmnesOutput { program, labels: _ } = asmnes::logical_assemble(program)?;
        let mut state = State::new_nrom128(program.clone(), program);
        state.run_instructions(n_instructions);
        Ok(state)
    }

    #[test]
    fn test_transfer_registers_by_label() -> Result<(), AsmnesError> {
        let state = run_program(3, &[
            INSTRL::INSTR(INSTR(LDA, IMM, U8(0x02))),
            INSTRL::INSTR(INSTR(STA, ABS, Label("VAR_A".to_string()))),
            INSTRL::INSTR(INSTR(LDX, ABS, Label("VAR_A".to_string()))),
            INSTRL::LABEL("VAR_A".to_string()),
            INSTRL::DIR(Directive::RS(1)),
        ])?;
        state.print_state();
        assert!(state.x == 0x02);
        Ok(())
    }
}
