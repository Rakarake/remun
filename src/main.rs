use shared::Opcode::*;
use shared::AddressingMode::*;
use remun::Operand::*;
use remun::INSTR;
use remun::State;

fn main() {
    let test_program: Vec<u8> = [
        INSTR(LDA, IMM, U8(0x02))
    ].iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat();
    let mut state = State {
        pc: 0,
        a: 0,
        x: 0,
        y: 0,
        sr: 0,
        sp: 0xFF,
        cycles: 0,
        ram: [0; 0x0800],
    };
    // Fill ram with test program
    for (i,ele) in test_program.iter().enumerate() {
        state.ram[i] = *ele;
    }
    state.run_one_instruction();
    println!("{:?}", state.a);
}

