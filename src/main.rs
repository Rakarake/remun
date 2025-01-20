mod instructions;

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

// Instructions
#[derive(Debug, PartialEq, Eq, Clone)]
struct Instruction {
    opcode: Opcode,
    addressing_mode: AddressingMode,
    //cycles: u8,
}

