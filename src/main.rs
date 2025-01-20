mod opcodes;
mod addressing_modes;
mod instructions;

use opcodes::Opcode;
use addressing_modes::AddressingMode;
use instructions::Instruction;

fn main() {
    use opcodes::Opcode::*;
    use addressing_modes::AddressingMode::*;
    use Operand::*;
    let test_program: Vec<u8> = [
        INSTR(LDA, IMM, U8(0x02))
    ].iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat();
    let mut state = State {
        pc: 0,
        a: 0,
        x: 0,
        y: 0,
        sr: 0,
        sp: 0xFF,
        cycles: 0,
        ram: [0; 0x0800],
    };
    // Fill ram with test program
    for (i,ele) in test_program.iter().enumerate() {
        state.ram[i] = *ele;
    }
    state.run_one_instruction();
    println!("{:?}", state.a);
}

/// Struct for easy NES program debugging.
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
    /// Program counter
    pc: u16,            
    /// Accumulator register
    a: u8,              
    /// X register
    x: u8,              
    /// Y register
    y: u8,              
    /// Status register: NV-BDIZC
    /// N  Negative
    /// V  Overflow
    /// -  ignored
    /// B  Break
    /// D  Decimal (unused on the NES)
    /// I  Interrupt (IRQ disable)
    /// Z  Zero
    /// C  Carry
    sr: u8,             
    /// Stack pointer
    sp: u8,             
    /// Number of cycles that have passed
    cycles: u64,        
    /// System RAM: $0000-$07FF, 2KiB
    ram: [u8; 0x0800],  
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
    }
    fn write(&mut self, address: u16, value: u8) {
        if address < 0x0800 {
            self.ram[address as usize] = value;
        } else {
            unimplemented!()
        }
    }
}

/// An addressing mode addresses memory, one of these.
#[derive(Debug, PartialEq, Eq)]
enum MemoryTarget {
    Address(u16),
    Accumulator,
    Impl,
}

