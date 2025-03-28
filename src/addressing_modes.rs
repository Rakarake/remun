use crate::MemoryTarget;
use crate::State;
use shared::AddressingMode;

// TODO implement number of cycles
/// Increases PC, returns the memory target/adress for opcode
/// to work on.
pub fn run(addressing_mode: AddressingMode, state: &mut State) -> MemoryTarget {
    use AddressingMode::*;
    use MemoryTarget::*;
    match addressing_mode {
        IMPL => {
            state.pc += 1;
            Impl
        }
        A => {
            state.pc += 1;
            Accumulator
        }
        IMM => {
            state.pc += 1;
            let a = Address(state.pc);
            state.pc += 1;
            a
        }
        ABS => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            let hi = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16 + ((hi as u16) << 8))
        }
        ABS_X => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16 + state.x as u16)
        }
        ABS_Y => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16 + state.y as u16)
        }
        REL => {
            state.pc += 1;
            let addr = state.pc;
            state.pc += 1;
            Address(addr)
        }
        ZPG => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16)
        }
        ZPG_X => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16 + state.x as u16)
        }
        ZPG_Y => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            Address(lo as u16 + state.y as u16)
        }
        IND => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            let hi = state.read(state.pc, false);
            state.pc += 1;
            let pointer = lo as u16 + ((hi as u16) << 8);
            Address(state.read(pointer, false) as u16 + ((state.read(pointer + 1, false) as u16) << 8))
        }
        X_IND => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            let pointer = (lo as u16) + (state.x as u16);
            Address(state.read(pointer, false) as u16 + ((state.read(pointer + 1, false) as u16) << 8))
        }
        IND_Y => {
            state.pc += 1;
            let lo = state.read(state.pc, false);
            state.pc += 1;
            let pointer = lo as u16;
            Address(
                state.read(pointer, false) as u16
                    + ((state.read(pointer + 1, false) as u16) << 8)
                    + state.y as u16,
            )
        }
        J => unimplemented!(),
    }
}
