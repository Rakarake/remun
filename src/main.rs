use asmnes::AsmnesError;
use asmnes::AsmnesOutput;
use shared::Opcode::*;
use shared::AddressingMode::*;
use asmnes::Operand::*;
use asmnes::INSTR;
use asmnes::INSTRL;
use remun::State;
use remun::Range;

// TODO make asmnes program struct, takes in ines or (prg, debug, char)? (no, depends on mappers)
// new_form_regions(regions, debug)

fn main() -> Result<(), AsmnesError> {
    let AsmnesOutput { program, labels } = asmnes::logical_assemble_plus(&[
        INSTRL::INSTR(INSTR(LDA, IMM, U8(0x02))),
        INSTRL::LABEL("HELLO_WORLD".to_string()),
        INSTRL::INSTR(INSTR(LDA, IMM, U8(0x02))),
        INSTRL::INSTR(INSTR(STA, ABS, Label("HELLO_WORLD".to_string()))),
        INSTRL::INSTR(INSTR(LDX, ABS, Label("HELLO_WORLD".to_string()))),
    ])?;
    let mut state = State::new_nrom128(program.clone(), program);
    println!("labels: {:?}", labels);
    state.run_one_instruction();
    println!("a: {}", state.a);
    state.run_one_instruction();
    state.run_one_instruction();
    state.run_one_instruction();
    state.print_state();
    println!("addr 2!? {}", state.read(2)); // should be HELLO_WORLD label
    println!("{:?}", state.x);
    Ok(())
}

