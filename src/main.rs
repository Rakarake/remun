use asmnes::AsmnesError;
use asmnes::AsmnesOutput;
use shared::Opcode::*;
use shared::AddressingMode::*;
use asmnes::Operand::*;
//use asmnes::INSTR;
//use asmnes::INSTRL;
use asmnes::Directive;
use remun::State;

mod visualizer;

// TODO make asmnes program struct, takes in ines or (prg, debug, char)? (no, depends on mappers)
// new_form_regions(regions, debug)

fn main() -> Result<(), AsmnesError> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    visualizer::main().unwrap();
    //let AsmnesOutput { program, labels } = asmnes::logical_assemble(&[
    //    INSTRL::INSTR(INSTR(LDA, IMM, U8(0x02))),
    //    INSTRL::INSTR(INSTR(STA, ABS, Label("VAR_A".to_string()))),
    //    INSTRL::INSTR(INSTR(LDX, ABS, Label("VAR_A".to_string()))),
    //    INSTRL::DIR(Directive::DS(2)),
    //    INSTRL::LABEL("VAR_A".to_string()),
    //    INSTRL::DIR(Directive::DS(4)),
    //    INSTRL::LABEL("VAR_B".to_string()),
    //])?;
    //let mut state = State::new_nrom128(program.clone(), program);
    //println!("labels: {:?}", labels);
    //state.run_one_instruction();
    //state.run_one_instruction();
    //state.run_one_instruction();
    //state.print_state();
    Ok(())
}

