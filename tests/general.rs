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
    #[test]
    fn test_transfer_registers_by_label() -> Result<(), AsmnesError> {
        let AsmnesOutput { program, labels: _ } = asmnes::logical_assemble(&[
            INSTRL::INSTR(INSTR(LDA, IMM, U8(0x02))),
            INSTRL::INSTR(INSTR(STA, ABS, Label("VAR_A".to_string()))),
            INSTRL::INSTR(INSTR(LDX, ABS, Label("VAR_A".to_string()))),
            INSTRL::LABEL("VAR_A".to_string()),
            INSTRL::DIR(Directive::RS(4)),
        ])?;
        let mut state = State::new_nrom128(program.clone(), program);
        state.run_one_instruction();
        state.run_one_instruction();
        state.run_one_instruction();
        assert!(state.x == 0x02);
        Ok(())
    }
}
