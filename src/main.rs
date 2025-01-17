macro_rules! another_nord_w {
    () => {
        5
    };
}


fn main() {
    println!("Hello, world!");
}

struct State {
    pc: u16,
    a: u8,
}

// Instruction lookup

// Opcode
enum Opcode {
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
    NOP,
    JAM,
}

impl Opcode {
    fn run(&self, state: &mut State) {
    }
}

// Addressing modes
enum AddressingMode {
    IMPL,  // nuh uh
    A,     // accumulator
    IMM,   // #FF
    REL,   // $FF
    ABS,   // $FFFF
    ABS_X, // $FFFF,X
    ABS_Y, // $FFFF,Y
    IND,   // ($FFFF)
    X_IND, // ($FF,X)
    IND_Y, // ($FF),Y
    ZPG,   // $FF
    ZPG_X, // $FF,X
    ZPG_Y, // $FF,Y
}

impl AddressingMode {
    fn run(&self, state: &mut State) {
    }
}

// Instructions
struct Instruction {
    opcode: Opcode,
    addressing_mode: AddressingMode,
    //cycles: u8,
}

//const INSTRUCTIONS: [Instruction; 256] = [
//];

const W: u8 = another_nord_w!();

macro_rules! tabalize {
    () => {
        
    };
}

mod instruction_stuff {
    use crate::Opcode::*;
    use crate::Opcode;
    use crate::AddressingMode::*;
    use crate::AddressingMode;

    const INSTRUCTIONS: [Instruction; 256] = [
        BRK IMPL , ORA X_IND , JAM     , SLO X_IND , NOP ZPG   , ORA ZPG   , ASL ZPG   , SLO ZPG   , PHP IMPL , ORA IMM   ,  ASL A    , ANC IMM   , NOP ABS   , ORA ABS   , ASL ABS   , SLO ABS   ,
        BPL REL  , ORA IND_Y , JAM     , SLO IND_Y , NOP ZPG_X , ORA ZPG_X , ASL ZPG_X , SLO ZPG_X , CLC IMPL , ORA ABS_Y ,  NOP IMPL , SLO ABS_Y , NOP ABS_X , ORA ABS_X , ASL ABS_X , SLO ABS_X ,
        JSR ABS  , AND X_IND , JAM     , RLA X_IND , BIT ZPG   , AND ZPG   , ROL ZPG   , RLA ZPG   , PLP IMPL , AND IMM   ,  ROL A    , ANC IMM   , BIT ABS   , AND ABS   , ROL ABS   , RLA ABS   ,
        BMI REL  , AND IND_Y , JAM     , RLA IND_Y , NOP ZPG_X , AND ZPG_X , ROL ZPG_X , RLA ZPG_X , SEC IMPL , AND ABS_Y ,  NOP IMPL , RLA ABS_Y , NOP ABS_X , AND ABS_X , ROL ABS_X , RLA ABS_X ,
        RTI IMPL , EOR X_IND , JAM     , SRE X_IND , NOP ZPG   , EOR ZPG   , LSR ZPG   , SRE ZPG   , PHA IMPL , EOR IMM   ,  LSR A    , ALR IMM   , JMP ABS   , EOR ABS   , LSR ABS   , SRE ABS   ,
        BVC REL  , EOR IND_Y , JAM     , SRE IND_Y , NOP ZPG_X , EOR ZPG_X , LSR ZPG_X , SRE ZPG_X , CLI IMPL , EOR ABS_Y ,  NOP IMPL , SRE ABS_Y , NOP ABS_X , EOR ABS_X , LSR ABS_X , SRE ABS_X ,
        RTS IMPL , ADC X_IND , JAM     , RRA X_IND , NOP ZPG   , ADC ZPG   , ROR ZPG   , RRA ZPG   , PLA IMPL , ADC IMM   ,  ROR A    , ARR IMM   , JMP IND   , ADC ABS   , ROR ABS   , RRA ABS   ,
        BVS REL  , ADC IND_Y , JAM     , RRA IND_Y , NOP ZPG_X , ADC ZPG_X , ROR ZPG_X , RRA ZPG_X , SEI IMPL , ADC ABS_Y ,  NOP IMPL , RRA ABS_Y , NOP ABS_X , ADC ABS_X , ROR ABS_X , RRA ABS_X ,
        NOP IMM  , STA X_IND , NOP IMM , SAX X_IND , STY ZPG   , STA ZPG   , STX ZPG   , SAX ZPG   , DEY IMPL , NOP IMM   ,  TXA IMPL , ANE IMM   , STY ABS   , STA ABS   , STX ABS   , SAX ABS   ,
        BCC REL  , STA IND_Y , JAM     , SHA IND_Y , STY ZPG_X , STA ZPG_X , STX ZPG_Y , SAX ZPG_Y , TYA IMPL , STA ABS_Y ,  TXS IMPL , TAS ABS_Y , SHY ABS_X , STA ABS_X , SHX ABS_Y , SHA ABS_Y ,
        LDY IMM  , LDA X_IND , LDX IMM , LAX X_IND , LDY ZPG   , LDA ZPG   , LDX ZPG   , LAX ZPG   , TAY IMPL , LDA IMM   ,  TAX IMPL , LXA IMM   , LDY ABS   , LDA ABS   , LDX ABS   , LAX ABS   ,
        BCS REL  , LDA IND_Y , JAM     , LAX IND_Y , LDY ZPG_X , LDA ZPG_X , LDX ZPG_Y , LAX ZPG_Y , CLV IMPL , LDA ABS_Y ,  TSX IMPL , LAS ABS_Y , LDY ABS_X , LDA ABS_X , LDX ABS_Y , LAX ABS_Y ,
        CPY IMM  , CMP X_IND , NOP IMM , DCP X_IND , CPY ZPG   , CMP ZPG   , DEC ZPG   , DCP ZPG   , INY IMPL , CMP IMM   ,  DEX IMPL , SBX IMM   , CPY ABS   , CMP ABS   , DEC ABS   , DCP ABS   ,
        BNE REL  , CMP IND_Y , JAM     , DCP IND_Y , NOP ZPG_X , CMP ZPG_X , DEC ZPG_X , DCP ZPG_X , CLD IMPL , CMP ABS_Y ,  NOP IMPL , DCP ABS_Y , NOP ABS_X , CMP ABS_X , DEC ABS_X , DCP ABS_X ,
        CPX IMM  , SBC X_IND , NOP IMM , ISC X_IND , CPX ZPG   , SBC ZPG   , INC ZPG   , ISC ZPG   , INX IMPL , SBC IMM   ,  NOP IMPL , USBC IMM  , CPX ABS   , SBC ABS   , INC ABS   , ISC ABS   ,
        BEQ REL  , SBC IND_Y , JAM     , ISC IND_Y , NOP ZPG_X , SBC ZPG_X , INC ZPG_X , ISC ZPG_X , SED IMPL , SBC ABS_Y ,  NOP IMPL , ISC ABS_Y , NOP ABS_X , SBC ABS_X , INC ABS_X , ISC ABS_X ,
    ];
}

