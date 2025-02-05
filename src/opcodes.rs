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
            match opcode {
                LDA => {
                    let val = state.read(addr);
                    state.a = val;
                    state.set_flag(flags::Z, val == 0);
                },
                LDX => {
                    let val = state.read(addr);
                    state.x = val;
                    state.set_flag(flags::Z, val == 0);
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
                _ => unimplemented!()
            }
        },
    }
}

