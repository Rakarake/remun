use crate::MemoryTarget;
use crate::State;

// Addressing modes
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AddressingMode {
    IMPL,  // nuh uh
    A,     // accumulator
    IMM,   // #$FF
    REL,   // $FF
    ABS,   // $LOHI
    ABS_X, // $LOHI,X
    ABS_Y, // $LOHI,Y
    IND,   // ($LOHI)
    X_IND, // ($LO,X)
    IND_Y, // ($LO),Y
    ZPG,   // $LO
    ZPG_X, // $LO,X
    ZPG_Y, // $LO,Y
    J,     // jam :(
}

impl AddressingMode {
    // TODO implement number of cycles
    /// Increases PC, returns the memory target/adress for opcode
    /// to work on.
    pub fn run(&self, state: &mut State) -> MemoryTarget {
        use AddressingMode::*;
        use MemoryTarget::*;
        match self {
            IMPL => Impl,
            A => Accumulator,
            IMM => {
                state.pc += 1;
                let a = Address(state.pc);
                state.pc += 1;
                a
            },
            ABS => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let hi = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16 + ((hi as u16) << 8))
            },
            ABS_X => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16 + state.x as u16)
            },
            ABS_Y => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16 + state.y as u16)
            },
            REl => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(state.pc + lo as u16)
            },
            ZPG => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16)
            },
            ZPG_X => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16 + state.x as u16)
            },
            ZPG_Y => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                Address(lo as u16 + state.y as u16)
            },
            IND => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let hi = state.read(state.pc);
                state.pc += 1;
                let pointer = (lo as u16 + (((hi as u16) << 0x10)));
                Address(state.read(pointer) as u16 + ((state.read(pointer + 1) as u16) << 0x10))
            },
            X_IND => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let pointer = (lo as u16 + state.x as u16);
                Address(state.read(pointer) as u16 + ((state.read(pointer + 1) as u16) << 0x10))
            },
            IND_Y => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let pointer = lo as u16;
                Address(state.read(pointer) as u16 + ((state.read(pointer + 1) as u16) << 0x10) + state.y as u16)
            },
            J => unimplemented!(),
        }
    }
}
