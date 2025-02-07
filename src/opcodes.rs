use shared::Opcode;
use crate::State;
use crate::MemoryTarget;
use shared::flags;

/// Expects pc to be at next instruction
pub fn run(opcode: Opcode, state: &mut State, memory_target: MemoryTarget) {
    use Opcode::*;
    use crate::MemoryTarget::*;
    match memory_target {
        Address(addr) => {
            let old = state.read(addr);
            match opcode {
                LDA => {
                    let val = old;
                    state.a = val;
                    new_value(state, val);
                },
                LDX => {
                    let val = old;
                    state.x = val;
                    new_value(state, val);
                },
                LDY => {
                    let val = old;
                    state.y = val;
                    new_value(state, val);
                },
                ROL => {
                    let (val, new_c) = old.overflowing_shl(1);
                    let val = val | state.get_flag(flags::C) as u8;
                    state.write(addr, val);
                    state.set_flag(flags::C, new_c);
                    new_value(state, val);
                },
                STA => {
                    state.write(addr, state.a);
                },
                _ => unimplemented!()
            }
        },
        Accumulator => {
            match opcode {
                _ => unimplemented!()
            }
        },
        Impl => {
            match opcode {
                _ => unimplemented!("nooo: {:?}", opcode)
            }
        },
    }
}

/// Common flag operations for when updating some value
fn new_value(state: &mut State, val: u8) {
    state.set_flag(flags::Z, val == 0);
    state.set_flag(flags::N, val & (1<<7) != 0);
}

