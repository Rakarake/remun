use AddressingMode::*;
use Opcode::*;
use std::{collections::HashMap, error::Error, fmt, fs::File, io::{self, BufReader, Read}, path::Path, str::FromStr};
use strum::IntoEnumIterator;

/// Simple range struct for range checking.
/// `0`: inclusive, `1`: exclusive.
#[derive(Clone, Copy, Debug)]
pub struct Range(pub u16, pub u16);
impl Range {
    pub fn contains(&self, value: u16) -> bool {
        value >= self.0 && value < self.1
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06X}-{:#06X}", self.0, self.1)
    }
}


/// In number of bytes.
/// Banks can have different meanings in different context,
/// this is the minimal size of a ROM region that can
/// be mapped (to my knowledge). This is the size that is used
/// for the banks in the assembler.
pub const BANK_SIZE: usize = 1024 * 8;

/// Representation of an iNES file (not NES 2.0 just yet :))
pub struct Ines {
    /// Size of PRG ROM in 16KiB units.
    pub inesprg: u16,
    /// Size of CHR ROM in 8KiB units.
    pub ineschr: u16,
    /// Vertically mirrored (1), Horizontally mirrored (0).
    pub mirroring: u16,
    /// The iNES mapper index, does not fully describe the hardware.
    pub mapper: u16,
    /// The rest of the iNES file, PRG first then CHR.
    pub banks: Vec<u8>,
    /// Debug information.
    pub labels: HashMap<String, u16>,
}

/// Error when reading an INES file.
#[derive(Debug)]
pub enum InesError {
  ParseError(InesParseError),
  IO(io::Error)
}

/// Error when parsing an INES file.
#[derive(Debug, strum_macros::Display)]
pub enum InesParseError {
    InvalidHeader,
    FileInvalidLength,
}

impl From<io::Error> for InesError {
    fn from(value: io::Error) -> Self {
        InesError::IO(value)
    }
}

impl From<InesParseError> for InesError {
    fn from(value: InesParseError) -> Self {
        InesError::ParseError(value)
    }
}

impl fmt::Display for InesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}", match self {
                InesError::IO(e) => {
                    format!("{}", e)
                },
                InesError::ParseError(e) => {
                    format!("{}", e)
                },
            }
        )
    }
}

impl std::error::Error for InesError {}

impl Ines {
    /// Reads INES file.
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, InesError> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        let mut header: [u8; 16] = [0; 16];
        reader.read_exact(&mut header)?;
        if &header[0..4] != b"NES\x1a" {
            return Err(InesError::ParseError(InesParseError::InvalidHeader))
        }
        let inesprg = header[4] as u16;
        let ineschr = header[5] as u16;
        let mirroring = (header[6] & 1) as u16;
        let mapper = (header[6] >> 4) as u16;
        let mut banks = Vec::new();
        let data_len = reader.read_to_end(&mut banks)?;
        let expected_size = inesprg as usize * 1024 * 16 + ineschr as usize * 1024 * 8;
        if data_len != expected_size {
            return Err(InesError::ParseError(InesParseError::FileInvalidLength))
        }
        Ok(Ines {
            inesprg,
            ineschr,
            mirroring,
            mapper,
            labels: HashMap::new(),
            banks,
        })
    }
}

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

impl From<u8> for AddressingMode {
    fn from(value: u8) -> Self {
        CODEPOINTS[value as usize].addressing_mode.clone()
    }
}

impl AddressingMode {
    pub fn arity(&self) -> usize {
        use AddressingMode::*;
        // so, how many bytes does JAM instructions need, none? infinate?
        match self {
            IMPL | A | J => 0,
            IMM | ZPG | ZPG_X | ZPG_Y | REL | X_IND | IND_Y => 1,
            ABS | ABS_X | ABS_Y | IND => 2,
        }
    }
}

pub mod flags {
    /// Negative flag: is bit7 1?
    pub const N: u8 = 1 << 7;
    /// Overflow
    pub const V: u8 = 1 << 6;
    //pub const 1
    //pub const b
    /// Decimal mode (unused)
    pub const D: u8 = 1 << 3;
    /// Interrupt inhibit: disables maskable interrupts
    pub const I: u8 = 1 << 2;
    /// Zero: is the reuslt 0
    pub const Z: u8 = 1 << 1;
    /// Carry: does add operation carry over?
    pub const C: u8 = 1 << 0;
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

// TODO capitalization should not matter
impl FromStr for Opcode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CODEPOINTS
            .iter()
            .find_map(
                |Codepoint {
                     opcode,
                     addressing_mode: _,
                 }| {
                    if format!("{}", opcode) == s {
                        Some(opcode)
                    } else {
                        None
                    }
                },
            )
            .cloned()
            .ok_or(())
    }
}

/// All opcodes, implements Display.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display, strum_macros::EnumIter)]
pub enum Opcode {
    /// Add with carry
    ADC,
    /// Logical And
    AND,
    /// Arithmatic Shift Left
    ASL,
    /// Branch on Carry Clear
    BCC,
    /// Branch on Carry Set
    BCS,
    /// Branch on Equals
    BEQ,
    /// Test Bits
    BIT,
    /// Branch on Minus
    BMI,
    /// Branch on Not Equals
    BNE,
    /// Branch on Plus
    BPL,
    /// Break
    BRK,
    /// Branch on Overflow Clear
    BVC,
    /// Branch on Overflow Set
    BVS,
    /// Clear Carry
    CLC,
    /// Clear Decimal Mode
    CLD,
    /// Clear Interrupt Disable
    CLI,
    /// Clear Overflow
    CLV,
    /// Compare with accumulator
    CMP,
    /// Compare with X
    CPX,
    /// Compare with Y
    CPY,
    /// Decrement memory by one
    DEC,
    /// Decrement X by one
    DEX,
    /// Decrement Y by one
    DEY,
    /// Exclusive-OR with accumulator
    EOR,
    /// Increment memory by one
    INC,
    /// Increment X by one
    INX,
    /// Increment Y oy one
    INY,
    /// Jump
    JMP,
    /// Jump and save return address on the stack
    JSR,
    /// Load Accumulator
    LDA,
    /// Load X
    LDX,
    /// Load Y
    LDY,
    /// Shift one bit right
    LSR,
    /// No Operation
    NOP,
    /// Or memory with Accumulator
    ORA,
    /// Push Accumulator on the stack
    PHA,
    /// Push Status register to the stack
    PHP,
    /// Pull Accumulator from stack
    PLA,
    /// Pull Status register from the stack
    PLP,
    /// Rotate left
    ROL,
    /// Rotate right
    ROR,
    /// Return from Interrupt
    RTI,
    /// Return from Subroutine
    RTS,
    /// Subtract from Accumulator with borrow (inverted carry)
    SBC,
    /// Set Carry
    SEC,
    /// Set Decimal
    SED,
    /// Set Interrupt Disable
    SEI,
    /// Store Accumulator
    STA,
    /// Store X
    STX,
    /// Store Y
    STY,
    /// Transfer A to X
    TAX,
    /// Transfer A to Y
    TAY,
    /// Transfer Stack Pointer to X
    TSX,
    // Transfer X to A
    TXA,
    /// Transfer X to Stack pointer
    TXS,
    /// Transfer Y to A
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

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        CODEPOINTS[value as usize].opcode.clone()
    }
}

pub fn opcode_iter() -> OpcodeIter {
    Opcode::iter()
}

/// Gets all available addressing modes for opcode.
pub fn opcode_addressing_modes(o: &Opcode) -> Vec<AddressingMode> {
    CODEPOINTS
        .iter()
        .filter_map(
            |Codepoint {
                 opcode,
                 addressing_mode,
             }| {
                if *o == *opcode {
                    Some(addressing_mode)
                } else {
                    None
                }
            },
        )
        .cloned()
        .collect()
}

/// A codepoint is an opcode and a addressing-mode, in other words
/// an instruction without the operand.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Codepoint {
    pub opcode: Opcode,
    pub addressing_mode: AddressingMode,
    //cycles: u8,
}

macro_rules! tabalize {
    ($($x:expr,$y:expr ); *;) => {
        [ $(Codepoint { opcode: $x, addressing_mode: $y },)* ]
    };
}

// Zoom out to see properly :)
/// The entire 6502 instuction set.
pub const CODEPOINTS: [Codepoint; 256] = tabalize! [
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
