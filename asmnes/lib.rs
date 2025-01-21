use shared::Opcode;
use shared::AddressingMode;
use shared::Instruction;
use shared::INSTRUCTIONS;

pub const MORG: u32 = 3;

pub fn assemble(input: String) -> Vec<u8> {
    
    return vec![];
}

/// Assembles from the handy INSTR
pub fn logical_assemble(instructions: &[INSTR]) -> Vec<u8> {
    instructions.iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat()
}

pub enum Operand {
    No,
    U8(u8),
    U16(u16),
}

/// Struct for easy NES program debugging.
pub struct INSTR(pub Opcode, pub AddressingMode, pub Operand);

impl INSTR {
    pub fn get_bytes(&self) -> Vec<u8> {
        let INSTR(op,a,operand) = self;
        if let Some(index) = 
            INSTRUCTIONS.iter().position(|Instruction { opcode, addressing_mode }| {
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


