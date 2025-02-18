#[cfg(test)]
mod test_general {
    use asmnes::AsmnesError;
    use asmnes::AsmnesOutput;
    use shared::Opcode::*;
    use shared::AddressingMode::*;
    use asmnes::Operand::*;
    use asmnes::Instruction;
    use asmnes::Line;
    use asmnes::Directive;
    use remun::State;
    use asmnes::Bank;
    use shared::Range;

    /// Helper to run a simple program.
    fn run_program(n_instructions: u64, program: &[Line]) -> Result<State, AsmnesError> {
        // NROM 128
        // TODO fix chr data and move this
        let banks = vec![
            Bank { data: vec![0; 4000], ranges: vec![Range(0x8000,0xBFFF), Range(0xC000,0xFFFF)] },
            Bank { data: vec![0; 0], ranges: vec![Range(0x0000,0x0000)] },
        ];
        let AsmnesOutput { banks, labels: _ } = asmnes::logical_assemble(program, banks)?;
        let mut state = State::new_nrom128(banks[0].data.clone(), banks[1].data.clone());
        state.run_instructions(n_instructions);
        Ok(state)
    }

    #[test]
    fn test_transfer_registers_by_label() -> Result<(), AsmnesError> {
        let state = run_program(6, &[
            // RAM
            Line::Directive(Directive::Org(0x0000)),
            Line::Directive(Directive::Ds(1)),
            Line::Label("VAR_A".to_string()),
            Line::Directive(Directive::Ds(1)),
            Line::Label("VAR_B".to_string()),
            Line::Directive(Directive::Ds(1)),

            // ROM
            Line::Directive(Directive::Bank(0)),
            Line::Directive(Directive::Org(0xC000)),

            // Load 2 value into VAR_A
            Line::Instruction(Instruction(LDA, IMM, U8(0x02))),
            Line::Instruction(Instruction(STA, ABS, Label("VAR_A".to_string()))),

            // Load 4 value into VAR_B
            Line::Instruction(Instruction(LDA, IMM, U8(0x04))),
            Line::Instruction(Instruction(STA, ABS, Label("VAR_B".to_string()))),

            // Load VAR_A and VAR_B into X and Y respectively
            Line::Instruction(Instruction(LDX, ABS, Label("VAR_A".to_string()))),
            Line::Instruction(Instruction(LDY, ABS, Label("VAR_B".to_string()))),
        ])?;
        state.print_state();
        assert!(state.x == 0x02);
        assert!(state.y == 0x04);
        Ok(())
    }
}
