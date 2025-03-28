use crate::MemoryTarget;
use crate::State;
use shared::Opcode;
use shared::flags;

// Thanks https://www.masswerk.at/6502/6502_instruction_set.html, made this
// a plesant experience!

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
            macro_rules! store_register {
                ($reg:expr) => {{
                    state.write(addr, $reg);
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
            macro_rules! bitwise {
                ($op:tt) => {{
                    let val = state.a $op state.read(addr, false);
                    state.a = val;
                    new_value(state, val);
                }};
            }
            macro_rules! addsub {
                ($op:ident, $carry:expr) => {{
                    let arg1 = state.a;
                    let arg2 = state.read(addr, false);
                    let old_c = $carry;

                    // Get c/carry (unsigned overflow) and the result
                    let (val, c) = arg1.$op(arg2);
                    let (val, c_2) = val.$op(old_c as u8);

                    // Get v/overflow (signed overflow)
                    let (val_test, v) = (arg1 as i8).$op(arg2 as i8);
                    let (_, v_2) = val_test.$op(old_c as i8);

                    state.a = val;
                    state.set_flag(flags::C, c | c_2);
                    state.set_flag(flags::V, v | v_2);
                    new_value(state, val);
                }};
            }
            macro_rules! incdec {
                ($by_what:tt) => {{
                    let val = state.read(addr, false);
                    state.write(addr, val.$by_what(1));
                    new_value(state, val);
                }};
            }
            match opcode {
                // Arithmetic Instructions
                ADC => addsub!(overflowing_add, state.get_flag(flags::C)),
                SBC => addsub!(overflowing_sub, !state.get_flag(flags::C)),

                // Bitwise operations
                AND => bitwise!(&),
                ORA => bitwise!(|),
                EOR => bitwise!(^),

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
                STA => store_register!(state.a),
                STX => store_register!(state.x),
                STY => store_register!(state.y),

                // Increments / Decrements
                DEC => incdec!(wrapping_sub),
                INC => incdec!(wrapping_add),

                o => unimplemented!("{o}"),
            }
        }
        Accumulator => match opcode {
            // Shift Instructions
            _ => unimplemented!("opcode not implemented: {:?}", opcode),
        },
        Impl => {
            macro_rules! set_flag {
                ($flag:expr, $val:expr) => {{
                    state.set_flag($flag, $val);
                }};
            }
            macro_rules! transfer {
                ($src:expr, $dst:expr) => {{
                    let val = $src;
                    $dst = val;
                    new_value(state, val);
                }};
            }
            macro_rules! push {
                ($what:expr) => {{
                    state.write(state.sp as u16 + 0x0100, $what);
                    let (new_pos, _) = state.sp.overflowing_sub(1);
                    state.sp = new_pos;
                }};
            }
            macro_rules! pull {
                ($what:expr) => {{
                    let (new_pos, _) = state.sp.overflowing_add(1);
                    state.sp = new_pos;
                    $what = state.read(state.sp as u16 + 0x0100, false);
                }};
            }
            macro_rules! incdecxy {
                ($reg:expr, $by_what:tt) => {{
                    let val = $reg.$by_what(1);
                    $reg = val;
                    new_value(state, val);
                }};
            }
            match opcode {
                // Stack Instructions
                PHA => push!(state.a),
                PHP => push!(state.sr | (1 << 5) | flags::B),
                PLA => pull!(state.a),
                PLP => pull!(state.sr),

                // Transfer Instructions
                TAX => transfer!(state.a, state.x),
                TAY => transfer!(state.a, state.y),
                TXA => transfer!(state.x, state.a),
                TYA => transfer!(state.y, state.a),
                TSX => transfer!(state.sp, state.x),
                TXS => transfer!(state.x, state.sp),

                // Flag instructions
                CLC => set_flag!(flags::C, false),
                SEC => set_flag!(flags::C, true),
                CLD => set_flag!(flags::D, false),
                SED => set_flag!(flags::D, true),
                CLV => set_flag!(flags::V, false),
                CLI => set_flag!(flags::I, false),
                SEI => set_flag!(flags::I, true),


                // Increments / Decrements
                DEX => incdecxy!(state.x, wrapping_sub),
                INX => incdecxy!(state.x, wrapping_add),
                DEY => incdecxy!(state.y, wrapping_sub),
                INY => incdecxy!(state.y, wrapping_add),

                NOP => {},
                _ => unimplemented!("opcode not implemented: {:?}", opcode),
            }
        }
    }
}

/// Common flag operations for when updating some value..
fn new_value(state: &mut State, val: u8) {
    state.set_flag(flags::Z, val == 0);
    state.set_flag(flags::N, val & (1 << 7) != 0);
}
