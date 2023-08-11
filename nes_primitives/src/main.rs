#[derive(Debug, Clone)]
enum Operand {
    // Different adressing modes
    Implied,               //
    Immediate(u8),         //#$44
    ZeroPage(u8),          //$44
    ZeroPageX(u8),         //$44,X
    Absolute(u16),         //$4400
    AbsoluteX(u16),        //$4400,X
    AbsoluteY(u16),        //$4400,Y
    IndirectX(u8),         //($44,X)
    IndirectY(u8),         //($44),Y
    Relative(RelativeVal), //$44
    Accumulator,           //A
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

