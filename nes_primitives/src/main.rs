#[derive(Debug, Clone)]
enum Operand {
    // Different adressing modes
    Implied,               //
    Immediate(Value<u8>),  //#$44
    ZeroPage(Value<u8>),   //$44
    ZeroPageX(Value<u8>),  //$44,X
    Absolute(Value<u16>),  //$4400
    AbsoluteX(Value<u16>), //$4400,X
    AbsoluteY(Value<u16>), //$4400,Y
    IndirectX(Value<u8>),  //($44,X)
    IndirectY(Value<u8>),  //($44),Y
    Relative(RelativeVal), //$44
    Accumulator,           //A
}

// A value of an operand
#[derive(Debug, Clone)]
enum Value<T> {
    Number(T),
    Label(String),
}

// Used for the relative addressing mode, a label points to a 16bit address,
// it is converted to a relative 8bit number later.
#[derive(Debug, Clone)]
enum RelativeVal {
    Number(u8),
    Label(String),
}

#[derive(Debug, Clone)]
enum Opcode {
    LDA,
    STA,
    NOP,
    BNE,
    ADC,
    AND,
    ASL,
    LSR,
    ROL,
    ROR,
    BIT,
    BRK,
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BEQ,
    CMP,
    CPX,
    CPY,
    DEC,
    INC,
    EOR,
}

