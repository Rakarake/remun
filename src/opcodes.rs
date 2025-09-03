use crate::MemoryTarget;
use crate::State;
use shared::Opcode;
use shared::flags;
use log::debug;

// Thanks https://www.masswerk.at/6502/6502_instruction_set.html, made this
// a plesant experience!

/// Shift operation, right indicates bitshift to the right, left otherwise.
/// Rotate indicates if the old carry is placed as the newly shifted in bit.
/// If addr is None, the shift will be done on the accumulator.
fn shift(state: &mut State, addr: Option<u16>, right: bool, rotate: bool) {
    // read value to be shifted
    let old = if let Some(addr) = addr {
        state.read(addr, false)
    } else {
        state.a
    };
    // find the bit that will be outshifted (new carry)
    let new_c = old & (1 << if right {0} else {7});
    // shift
    let val = if right {old >> 1} else {old << 1}
        | if rotate {(state.get_flag(flags::C) as u8) << if right {7} else {0}} else {0};
    // write result
    if let Some(addr) = addr {
        state.write(addr, val);
    } else {
        state.a = val;
    }
    // set carry
    state.set_flag(flags::C, new_c != 0);
    new_value(state, val);
}

/// add: use addition, otherwise subtraction
fn addsub(state: &mut State, sub: bool, arg1: u8, arg2: u8) -> u8 {
    // 'not' the carry if subtracting
    let old_c = state.get_flag(flags::C) ^ sub;

    // determine what operations to use to calculate unsigned carryover
    // and signed carryover for addition/subtraction respectively
    let op_u = if sub {|x: u8, y: u8| x.overflowing_sub(y)} else {|x: u8, y: u8| x.overflowing_add(y)};
    let op_i = if sub {|x: i8, y: i8| x.overflowing_sub(y)} else {|x: i8, y: i8| x.overflowing_add(y)};
    
    // Get c/carry (unsigned overflow) and the result
    let (val, c) = op_u(arg1, arg2);
    let (val, c_2) = op_u(val, old_c as u8);
    
    // Get v/overflow (signed overflow)
    let (val_test, v) = op_i(arg1 as i8, arg2 as i8);
    let (_, v_2) = op_i(val_test, old_c as i8);
    
    //state.a = val;
    state.set_flag(flags::C, c | c_2);
    state.set_flag(flags::V, v | v_2);
    new_value(state, val);
    val
}

fn incdec(state: &mut State, addr: u16, inc: bool) {
    let val = state.read(addr, false);
    state.write(addr, if inc {val.wrapping_add(1)} else {val.wrapping_sub(1)});
    new_value(state, val);
}

fn store_register(state: &mut State, addr: u16, reg: u8) {
    state.write(addr, reg);
}

fn branch(state: &mut State, addr: u16, cond: bool) {
    if cond {
        let old = state.read(addr, false);
        debug!("branching: {:?}", old as i8);
        state.pc = state.pc.wrapping_add_signed((old as i8) as i16);
    }
    debug!("not branching");
}

/// Expects pc to be at next instruction
pub fn run(opcode: Opcode, state: &mut State, memory_target: MemoryTarget) {
    use crate::MemoryTarget::*;
    use Opcode::*;
    macro_rules! push {
        ($what:expr) => {{
            state.write(state.sp as u16 + 0x0100, $what);
            dec_stack(state);
        }};
    }
    macro_rules! pull {
        ($what:expr) => {{
            inc_stack(state);
            $what = state.read(state.sp as u16 + 0x0100, false);
        }};
    }
    match memory_target {
        Address(addr) => {
            macro_rules! load_register {
                ($reg:expr) => {{
                    let val = state.read(addr, false);
                    $reg = val;
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
            match opcode {
                // Arithmetic Instructions
                //
    //let arg1 = state.a;
    //let arg2 = state.read(addr, false);
                ADC => {
                    let arg2 = state.read(addr, false);
                    state.a = addsub(state, false, state.a, arg2);
                },
                SBC => {
                    let arg2 = state.read(addr, false);
                    state.a = addsub(state, true, state.a, arg2);
                },
                CMP => {
                    let arg2 = state.read(addr, false);
                    addsub(state, true, state.a, arg2);
                }
                CPX => {
                    let arg2 = state.read(addr, false);
                    addsub(state, true, state.x, arg2);
                }
                CPY => {
                    let arg2 = state.read(addr, false);
                    addsub(state, true, state.y, arg2);
                }

                // The weird BIT instruction.
                // A AND M -> Z, M7 -> N, M6 -> V
                BIT => {
                    let arg = state.read(addr, false);
                    state.sr = arg & 0b11000000;
                    state.set_flag(flags::Z, state.a & arg == 0);
                }

                // Compare instructions (SUB without writing value back).

                // Bitwise operations
                AND => bitwise!(&),
                ORA => bitwise!(|),
                EOR => bitwise!(^),

                // Shift operations
                ROL => shift(state, Some(addr), false, true),
                ROR => shift(state, Some(addr), true, true),
                ASL => shift(state, Some(addr), false, false),
                LSR => shift(state, Some(addr), true, false),

                // Jump
                JMP => {
                    state.pc = addr;
                }

                // Branch Instructions
                BNE => branch(state, addr, !state.get_flag(flags::Z)),
                BEQ => branch(state, addr, state.get_flag(flags::Z)),

                BPL => branch(state, addr, !state.get_flag(flags::N)),
                BMI => branch(state, addr, state.get_flag(flags::N)),

                BVC => branch(state, addr, !state.get_flag(flags::V)),
                BVS => branch(state, addr, state.get_flag(flags::V)),

                BCC => branch(state, addr, !state.get_flag(flags::C)),
                BCS => branch(state, addr, state.get_flag(flags::C)),

                // Load Instructions
                LDA => load_register!(state.a),
                LDX => load_register!(state.x),
                LDY => load_register!(state.y),

                // Store Instructions
                STA => store_register(state, addr, state.a),
                STX => store_register(state, addr, state.x),
                STY => store_register(state, addr, state.y),

                // Increments / Decrements
                DEC => incdec(state, addr, false),
                INC => incdec(state, addr, true),

                // Subroutines
                JSR => {
                    push_pc(state);
                    state.pc = addr;
                }

                o => unimplemented!("address provided to opcode {o}, pc = {:#06X}", state.pc),
            }
        }
        Accumulator => match opcode {
            // Shift Instructions
            ROL => shift(state, None, false, true),
            ROR => shift(state, None, true, true),
            ASL => shift(state, None, false, false),
            LSR => shift(state, None, true, false),
            _ => unimplemented!("opcode {:?} not implemented for accumulator operation, pc = {:#06X}", opcode, state.pc),
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

                // Return from interrupt
                RTI => {
                    pull!(state.sr);
                    pull_pc(state);
                }

                // Return from subroutine
                RTS => {
                    pull_pc(state);
                    state.inc_pc();
                }

                // Break: initiate software interrupt
                BRK => {
                    // Skipping byte representing the reason for the interrupt
                    state.inc_pc();
                    push_pc(state);
                    push!(state.sr | flags::B | flags::I);
                    state.pc = state.read_u16(shared::vectors::BRK);
                }

                NOP => {},
                _ => unimplemented!("implied opcode not implemented: {:?}, pc = {:#06X}", opcode, state.pc),
            }
        }
    }
}

fn inc_stack(state: &mut State) {
    let (new_pos, _) = state.sp.overflowing_add(1);
    state.sp = new_pos;
}

fn dec_stack(state: &mut State) {
    let (new_pos, _) = state.sp.overflowing_add(1);
    state.sp = new_pos;
}

fn pull_pc(state: &mut State) {
    inc_stack(state);
    let pc_lo = state.read(state.sp as u16 | 0x0100, false);
    inc_stack(state);
    let pc_hi = state.read(state.sp as u16 | 0x0100, false);
    state.pc = ((pc_hi as u16) << 8) | pc_lo as u16;
}

fn push_pc(state: &mut State) {
    let pc_hi = (state.pc >> 8) as u8;
    state.write(state.sp as u16 | 0x0100, pc_hi);
    dec_stack(state);
    let pc_lo = state.pc as u8;
    state.write(state.sp as u16 | 0x0100, pc_lo);
    dec_stack(state);
}

/// Common flag operations for when updating some value..
fn new_value(state: &mut State, val: u8) {
    state.set_flag(flags::Z, val == 0);
    state.set_flag(flags::N, val & (1 << 7) != 0);
}
