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
            state.inc_pc();
            Impl
        }
        A => {
            state.inc_pc();
            Accumulator
        }
        IMM => {
            state.inc_pc();
            let a = Address(state.pc);
            state.inc_pc();
            a
        }
        ABS => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let hi = state.read(state.pc, false);
            state.inc_pc();
            Address(lo as u16 + ((hi as u16) << 8))
        }
        ABS_X => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let hi = state.read(state.pc, false);
            state.inc_pc();
            // addr + X with carry-over
            let addr = lo as u16 + ((hi as u16) << 8);
            // TODO implement page-boundary extra cycle when low byte carries over
            let (addr, _) = addr.overflowing_add(state.x as u16);
            Address(addr)
        }
        ABS_Y => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let hi = state.read(state.pc, false);
            state.inc_pc();
            // addr + X with carry-over
            let addr = lo as u16 + ((hi as u16) << 8);
            // TODO implement page-boundary extra cycle when low byte carries over
            let (addr, _) = addr.overflowing_add(state.y as u16);
            Address(addr)
        }
        REL => {
            state.inc_pc();
            let addr = state.pc;
            state.inc_pc();
            Address(addr)
        }
        ZPG => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            Address(lo as u16)
        }
        ZPG_X => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            Address(lo as u16 + state.x as u16)
        }
        ZPG_Y => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            Address(lo as u16 + state.y as u16)
        }
        IND => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let hi = state.read(state.pc, false);
            state.inc_pc();
            let pointer = lo as u16 + ((hi as u16) << 8);
            Address(
                state.read(pointer, false) as u16 + ((state.read(pointer + 1, false) as u16) << 8),
            )
        }
        X_IND => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let pointer = (lo as u16) + (state.x as u16);
            Address(
                state.read(pointer, false) as u16 + ((state.read(pointer + 1, false) as u16) << 8),
            )
        }
        IND_Y => {
            state.inc_pc();
            let lo = state.read(state.pc, false);
            state.inc_pc();
            let pointer = lo as u16;
            Address(
                state.read(pointer, false) as u16
                    + ((state.read(pointer + 1, false) as u16) << 8)
                    + state.y as u16,
            )
        }
        J => unimplemented!("the jam addressing mode (illegal!)"),
    }
}
