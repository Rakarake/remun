/// Addressing modes
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AddressingMode {
    IMPL,
    /// A
    A,
    /// #$FF
    IMM,
    /// $FF
    REL,
    /// $LOHI
    ABS,
    /// $LOHI,X
    ABS_X,
    /// $LOHI,Y
    ABS_Y, 
    /// ($LOHI)
    IND,   
    /// ($LO,X)
    X_IND,
    /// ($LO),Y
    IND_Y,
    /// $LO
    ZPG,
    /// $LO,X
    ZPG_X,
    /// $LO,Y
    ZPG_Y,
    /// jam :(
    J,
}

impl AddressingMode {
    /// Get the length of an instruction using this addressing mode
    pub fn get_len(&self) -> u16 {
        match self {
            AddressingMode::IMPL => 1,
            AddressingMode::A => 1,
            AddressingMode::IMM => 2,
            AddressingMode::REL => 2,
            AddressingMode::ZPG => 2,
            AddressingMode::ZPG_X => 2,
            AddressingMode::ZPG_Y => 2,
            AddressingMode::ABS => 3,
            AddressingMode::ABS_X => 3,
            AddressingMode::ABS_Y => 3,
            AddressingMode::IND => 3,
            AddressingMode::X_IND => 2,
            AddressingMode::IND_Y => 2,
            AddressingMode::J => unimplemented!("illegal instruction!"),
        }
    }
}

/// All opcodes, implements Display
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display, strum_macros::EnumIter)]
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

pub fn opcode_iter() -> OpcodeIter {
    Opcode::iter()
}

// Instructions
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub addressing_mode: AddressingMode,
    //cycles: u8,
}

macro_rules! tabalize {
    ($($x:expr,$y:expr ); *;) => {
        [ $(Instruction { opcode: $x, addressing_mode: $y },)* ]
    };
}

use strum::IntoEnumIterator;
use Opcode::*;
use AddressingMode::*;

// Zoom out to see properly :)
pub const INSTRUCTIONS: [Instruction; 256] = tabalize! [
    BRK,IMPL ; ORA,X_IND ; JAM,J   ; SLO,X_IND ; NOP,ZPG   ; ORA,ZPG   ; ASL,ZPG   ; SLO,ZPG   ; PHP,IMPL ; ORA,IMM   ; ASL,A    ; ANC,IMM   ; NOP,ABS   ; ORA,ABS   ; ASL,ABS   ; SLO,ABS   ;
    BPL,REL  ; ORA,IND_Y ; JAM,J   ; SLO,IND_Y ; NOP,ZPG_X ; ORA,ZPG_X ; ASL,ZPG_X ; SLO,ZPG_X ; CLC,IMPL ; ORA,ABS_Y ; NOP,IMPL ; SLO,ABS_Y ; NOP,ABS_X ; ORA,ABS_X ; ASL,ABS_X ; SLO,ABS_X ;
    JSR,ABS  ; AND,X_IND ; JAM,J   ; RLA,X_IND ; BIT,ZPG   ; AND,ZPG   ; ROL,ZPG   ; RLA,ZPG   ; PLP,IMPL ; AND,IMM   ; ROL,A    ; ANC,IMM   ; BIT,ABS   ; AND,ABS   ; ROL,ABS   ; RLA,ABS   ;
    BMI,REL  ; AND,IND_Y ; JAM,J   ; RLA,IND_Y ; NOP,ZPG_X ; AND,ZPG_X ; ROL,ZPG_X ; RLA,ZPG_X ; SEC,IMPL ; AND,ABS_Y ; NOP,IMPL ; RLA,ABS_Y ; NOP,ABS_X ; AND,ABS_X ; ROL,ABS_X ; RLA,ABS_X ;
    RTI,IMPL ; EOR,X_IND ; JAM,J   ; SRE,X_IND ; NOP,ZPG   ; EOR,ZPG   ; LSR,ZPG   ; SRE,ZPG   ; PHA,IMPL ; EOR,IMM   ; LSR,A    ; ALR,IMM   ; JMP,ABS   ; EOR,ABS   ; LSR,ABS   ; SRE,ABS   ;
    BVC,REL  ; EOR,IND_Y ; JAM,J   ; SRE,IND_Y ; NOP,ZPG_X ; EOR,ZPG_X ; LSR,ZPG_X ; SRE,ZPG_X ; CLI,IMPL ; EOR,ABS_Y ; NOP,IMPL ; SRE,ABS_Y ; NOP,ABS_X ; EOR,ABS_X ; LSR,ABS_X ; SRE,ABS_X ;
    RTS,IMPL ; ADC,X_IND ; JAM,J   ; RRA,X_IND ; NOP,ZPG   ; ADC,ZPG   ; ROR,ZPG   ; RRA,ZPG   ; PLA,IMPL ; ADC,IMM   ; ROR,A    ; ARR,IMM   ; JMP,IND   ; ADC,ABS   ; ROR,ABS   ; RRA,ABS   ;
    BVS,REL  ; ADC,IND_Y ; JAM,J   ; RRA,IND_Y ; NOP,ZPG_X ; ADC,ZPG_X ; ROR,ZPG_X ; RRA,ZPG_X ; SEI,IMPL ; ADC,ABS_Y ; NOP,IMPL ; RRA,ABS_Y ; NOP,ABS_X ; ADC,ABS_X ; ROR,ABS_X ; RRA,ABS_X ;
    NOP,IMM  ; STA,X_IND ; NOP,IMM ; SAX,X_IND ; STY,ZPG   ; STA,ZPG   ; STX,ZPG   ; SAX,ZPG   ; DEY,IMPL ; NOP,IMM   ; TXA,IMPL ; ANE,IMM   ; STY,ABS   ; STA,ABS   ; STX,ABS   ; SAX,ABS   ;
    BCC,REL  ; STA,IND_Y ; JAM,J   ; SHA,IND_Y ; STY,ZPG_X ; STA,ZPG_X ; STX,ZPG_Y ; SAX,ZPG_Y ; TYA,IMPL ; STA,ABS_Y ; TXS,IMPL ; TAS,ABS_Y ; SHY,ABS_X ; STA,ABS_X ; SHX,ABS_Y ; SHA,ABS_Y ;
    LDY,IMM  ; LDA,X_IND ; LDX,IMM ; LAX,X_IND ; LDY,ZPG   ; LDA,ZPG   ; LDX,ZPG   ; LAX,ZPG   ; TAY,IMPL ; LDA,IMM   ; TAX,IMPL ; LXA,IMM   ; LDY,ABS   ; LDA,ABS   ; LDX,ABS   ; LAX,ABS   ;
    BCS,REL  ; LDA,IND_Y ; JAM,J   ; LAX,IND_Y ; LDY,ZPG_X ; LDA,ZPG_X ; LDX,ZPG_Y ; LAX,ZPG_Y ; CLV,IMPL ; LDA,ABS_Y ; TSX,IMPL ; LAS,ABS_Y ; LDY,ABS_X ; LDA,ABS_X ; LDX,ABS_Y ; LAX,ABS_Y ;
    CPY,IMM  ; CMP,X_IND ; NOP,IMM ; DCP,X_IND ; CPY,ZPG   ; CMP,ZPG   ; DEC,ZPG   ; DCP,ZPG   ; INY,IMPL ; CMP,IMM   ; DEX,IMPL ; SBX,IMM   ; CPY,ABS   ; CMP,ABS   ; DEC,ABS   ; DCP,ABS   ;
    BNE,REL  ; CMP,IND_Y ; JAM,J   ; DCP,IND_Y ; NOP,ZPG_X ; CMP,ZPG_X ; DEC,ZPG_X ; DCP,ZPG_X ; CLD,IMPL ; CMP,ABS_Y ; NOP,IMPL ; DCP,ABS_Y ; NOP,ABS_X ; CMP,ABS_X ; DEC,ABS_X ; DCP,ABS_X ;
    CPX,IMM  ; SBC,X_IND ; NOP,IMM ; ISC,X_IND ; CPX,ZPG   ; SBC,ZPG   ; INC,ZPG   ; ISC,ZPG   ; INX,IMPL ; SBC,IMM   ; NOP,IMPL ; USB, IMM  ; CPX,ABS   ; SBC,ABS   ; INC,ABS   ; ISC,ABS   ;
    BEQ,REL  ; SBC,IND_Y ; JAM,J   ; ISC,IND_Y ; NOP,ZPG_X ; SBC,ZPG_X ; INC,ZPG_X ; ISC,ZPG_X ; SED,IMPL ; SBC,ABS_Y ; NOP,IMPL ; ISC,ABS_Y ; NOP,ABS_X ; SBC,ABS_X ; INC,ABS_X ; ISC,ABS_X ;
];

