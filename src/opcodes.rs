use crate::State;
use crate::MemoryTarget;

// Opcode
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Opcode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,

    // Illegal opcodes
    ALR,
    ANC,
    ANE,
    ARR,
    DCP,
    ISC,
    LAS,
    LAX,
    LXA,
    RLA,
    RRA,
    SAX,
    SBX,
    SHA,
    SHX,
    SHY,
    SLO,
    SRE,
    TAS,
    USB,
    JAM,
}

impl Opcode {
    /// Expects pc to be at next instruction
    pub fn run(&self, state: &mut State, memory_target: MemoryTarget) {
        use Opcode::*;
        use crate::MemoryTarget::*;
        match memory_target {
            Address(addr) => {
                match self {
                    LDA => {
                        let val = state.read(addr);
                        state.a = val;
                        state.sr = (val == 0) as u8;
                    },
                    STA => {
                        state.write(addr, state.a);
                    },
                    _ => unimplemented!()
                }
            },
            Accumulator => {
                match self {
                    _ => unimplemented!()
                }
            },
            Impl => {
                match self {
                    _ => unimplemented!()
                }
            },
        }
    }
}

