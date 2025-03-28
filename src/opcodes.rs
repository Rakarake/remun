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
            macro_rules! branch {
                ($cond:expr) => {{
                    if $cond {
                        let old = state.read(addr, false);
                        state.pc = state.pc.wrapping_add_signed(old as i16);
                    }
                }}
            }
            macro_rules! load_register {
                ($reg:expr) => {{
                    let val = state.read(addr, false);
                    $reg = val;
                    new_value(state, val);
                }};
            }
            macro_rules! shift {
                ($right:expr, $rotate:expr) => {{
                    let old = state.read(addr, false);
                    let new_c = old & (1 << if $right {0} else {7});
                    let val = if $right {(old >> 1)} else {(old << 1)}
                        | if $rotate {(state.get_flag(flags::C) as u8) << if $right {7} else {0}} else {0};
                    state.write(addr, val);
                    state.set_flag(flags::C, new_c != 0);
                    new_value(state, val);
                }};
            }
            match opcode {
                // Arithmetic Instructions
                // A + M + C -> A, C
                ADC => {
                    let arg1 = state.a;
                    let arg2 = state.read(addr, false);
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
                AND => {
                    let old = state.read(addr, false);
                    let val = state.a & old;
                    state.a = val;
                    new_value(state, val);
                }

                // Shift operations
                ROL => shift!(false, true),
                ROR => shift!(true, true),
                ASL => shift!(false, false),
                LSR => shift!(true, false),

                // Jump
                JMP => {
                    state.pc = addr;
                }

                // Branch Instructions
                BNE => branch!(!state.get_flag(flags::Z)),
                BEQ => branch!(state.get_flag(flags::Z)),

                BPL => branch!(!state.get_flag(flags::N)),
                BMI => branch!(state.get_flag(flags::N)),

                BVC => branch!(!state.get_flag(flags::V)),
                BVS => branch!(state.get_flag(flags::V)),

                BCC => branch!(!state.get_flag(flags::C)),
                BCS => branch!(state.get_flag(flags::C)),

                // Load Instructions
                LDA => load_register!(state.a),
                LDX => load_register!(state.x),
                LDY => load_register!(state.y),

                // Store Instructions
                STA => state.write(addr, state.a),
                STX => state.write(addr, state.x),
                STY => state.write(addr, state.y),

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
