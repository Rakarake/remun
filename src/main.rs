
fn main() {
    use Opcode::*;
    use AddressingMode::*;
    use Operand::*;
    let test_program: Vec<u8> = [
        INSTR(LDA, IMM, U8(0x02))
    ].iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat();
    let mut state = State {
        /// Program Counter
        pc: 0,
        /// Accumulator
        a: 0,
        /// X
        x: 0,
        /// Y
        y: 0,
        /// Status register: NV-BDIZC
        /// N	Negative
        /// V	Overflow
        /// -	ignored
        /// B	Break
        /// D	Decimal (unused on the NES)
        /// I	Interrupt (IRQ disable)
        /// Z	Zero
        /// C	Carry
        sr: 0,
        /// Stack Pointer
        sp: 0xFF,
        /// Number of cycles that have passed
        cycles: 0,
        /// Built in ram
        ram: [0; 0x0800],
    };
    // Fill ram with test program
    for (i,ele) in test_program.iter().enumerate() {
        state.ram[i] = *ele;
    }
    state.run_one_instruction();
    println!("{:?}", state.a);
}

struct INSTR(Opcode, AddressingMode, Operand);

impl INSTR {
    fn get_bytes(&self) -> Vec<u8> {
        let INSTR(op,a,operand) = self;
        if let Some(index) = 
            instructions::INSTRUCTIONS.iter().position(|Instruction { opcode, addressing_mode }| {
                op == opcode && a == addressing_mode
            })
        {
            use Operand::*;
            match operand {
                No => vec![index as u8],
                U8(b) => vec![index as u8, *b],
                U16(bs) => {
                    let mut x = vec![index as u8];
                    x.extend_from_slice(&bs.to_be_bytes());
                    x
                },
            }
        }
        else {
            panic!("no such instruction")
        }
    }
}

enum Operand {
    No,
    U8(u8),
    U16(u16),
}

/// `0`: inclusive, `1`: exclusive
struct Range(u16, u16);
impl Range {
    fn contains(&self, value: u16) -> bool {
        value >= self.0 && value < self.1
    }
}

const RAM_RANGE: Range = Range(0x0000, 0x0100);

struct State {
    pc: u16,            // Program counter
    a: u8,              // Accumulator register
    x: u8,              // X register
    y: u8,              // Y register
    sr: u8,             // Status register
    sp: u8,             // Stack pointer
    cycles: u64,        // Number of cycles that have passed
    ram: [u8; 0x0800],  // System RAM: $0000-$07FF, 2KiB
}

impl State {
    fn run_one_instruction(&mut self) {
        let instr = self.read(self.pc);
        let Instruction { opcode, addressing_mode } = instructions::INSTRUCTIONS[instr as usize].clone();
        let memory_target = addressing_mode.run(self);
        opcode.run(self, memory_target);
    }

    fn read(&mut self, address: u16) -> u8 {
        if RAM_RANGE.contains(address) {
            return self.ram[address as usize];
        } else {
            unimplemented!()
        }
        0
    }
    fn write(&mut self, address: u16, value: u8) {
        if address < 0x0800 {
            self.ram[address as usize] = value;
        } else {
            unimplemented!()
        }
    }
}

// Instruction lookup

// Opcode
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
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
    USBC,
    USB,
    JAM,
}

impl Opcode {
    /// Expects pc to be at next instruction
    fn run(&self, state: &mut State, memory_target: MemoryTarget) {
        use Opcode::*;
        use MemoryTarget::*;
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

// Addressing modes
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
enum AddressingMode {
    IMPL,  // nuh uh
    A,     // accumulator
    IMM,   // #FF
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

#[derive(Debug, PartialEq, Eq)]
enum MemoryTarget {
    Address(u16),
    Accumulator,
    Impl,
}

impl AddressingMode {
    // TODO implement number of cycles
    /// Increases PC, returns the memory target/adress for opcode
    /// to work on.
    fn run(&self, state: &mut State) -> MemoryTarget {
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
                let pointer = (lo as u16 + (((hi as u16) << 0x10))) as usize;
                Address(state.ram[pointer] as u16 + ((state.ram[pointer + 1] as u16) << 0x10))
            },
            X_IND => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let pointer = (lo as u16 + state.x as u16) as usize;
                Address(state.ram[pointer] as u16 + ((state.ram[pointer + 1] as u16) << 0x10))
            },
            IND_Y => {
                state.pc += 1;
                let lo = state.read(state.pc);
                state.pc += 1;
                let pointer = lo as usize;
                Address(state.ram[pointer] as u16 + ((state.ram[pointer + 1] as u16) << 0x10) + state.y as u16)
            },
            J => unimplemented!(),
        }
    }
}

// Instructions
#[derive(Debug, PartialEq, Eq, Clone)]
struct Instruction {
    opcode: Opcode,
    addressing_mode: AddressingMode,
    //cycles: u8,
}

mod instructions {
    use crate::Opcode::*;
    use crate::Opcode;
    use crate::AddressingMode::*;
    use crate::AddressingMode;
    use crate::Instruction;

    macro_rules! tabalize {
        ($($x:expr,$y:expr ); *;) => {
            [ $(Instruction { opcode: $x, addressing_mode: $y },)* ]
        };
    }

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
}

