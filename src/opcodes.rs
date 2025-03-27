use crate::MemoryTarget;
use crate::State;
use shared::Opcode;
use shared::flags;

/// Expects pc to be at next instruction
pub fn run(opcode: Opcode, state: &mut State, memory_target: MemoryTarget) {
    use crate::MemoryTarget::*;
    use Opcode::*;
    match memory_target {
        Address(addr) => {
            let old = state.read(addr, false);
            match opcode {
                // A + M + C -> A, C
                ADC => {
                    let arg1 = state.a;
                    let arg2 = old;
                    let old_c = state.get_flag(flags::C);

                    // Get c/carry (unsigned overflow) and the result
                    let (val, c) = arg1.overflowing_add(arg2);
                    let (val, c_2) = val.overflowing_add(old_c as u8);

                    // Get v/overflow (signed overflow)
                    let (val_test, v) = (arg1 as i8).overflowing_add(arg2 as i8);
                    let (_, v_2) = val_test.overflowing_add(old_c as i8);

                    state.a = val;
                    state.set_flag(flags::C, c | c_2);
                    state.set_flag(flags::V, v | v_2);
                    new_value(state, val);
                }
                LDA => {
                    let val = old;
                    state.a = val;
                    new_value(state, val);
                }
                LDX => {
                    let val = old;
                    state.x = val;
                    new_value(state, val);
                }
                LDY => {
                    let val = old;
                    state.y = val;
                    new_value(state, val);
                }
                ROL => {
                    let (val, new_c) = old.overflowing_shl(1);
                    let val = val | state.get_flag(flags::C) as u8;
                    state.write(addr, val);
                    state.set_flag(flags::C, new_c);
                    new_value(state, val);
                }
                STA => {
                    state.write(addr, state.a);
                }
                o => unimplemented!("{o}"),
            }
        }
        Accumulator => match opcode {
            _ => unimplemented!("opcode not implemented: {:?}", opcode),
        },
        Impl => match opcode {
            NOP => {},
            CLC => { state.set_flag(flags::C, false); }
            SEI => { state.set_flag(flags::I, true); }
            _ => unimplemented!("opcode not implemented: {:?}", opcode),
        },
    }
}

/// Common flag operations for when updating some value..
fn new_value(state: &mut State, val: u8) {
    state.set_flag(flags::Z, val == 0);
    state.set_flag(flags::N, val & (1 << 7) != 0);
}
