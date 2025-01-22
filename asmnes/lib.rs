use shared::Opcode;
use shared::AddressingMode;
use shared::Instruction;
use shared::opcode_iter;
use shared::INSTRUCTIONS;
use std::fmt;

pub const MORG: u32 = 3;

#[derive(Debug)]
struct AsmnesError {
    line: String,
    cause: String,
}

impl fmt::Display for AsmnesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error on line: {}, cause: {}", self.line, self.cause)
    }
}

/// Does not produce the finalized binary, only a local byte array
pub fn simple_assemble(input: &str) -> Vec<u8> {
    logical_assemble(&second_pass(&first_pass(input)))
}

/// Lexing
fn first_pass(input: &str) -> Vec<INSTR> {
    let mut output: Vec<INSTR> = Vec::new();
    let mut reader: Vec<char> = Vec::new();
    input.lines().for_each(|l| {
        // TODO do sanity checks about indentation and comments
        let mut words = l.split_whitespace();
        if let Some(word) = words.next() {
            // Is an opcode
            if let Some(o) = opcode_iter().find_map(|o| if o.to_string() == word { Some(o) } else { None }) {
                // Expecting addressing mode
            }
            // Is a comment
            if word.starts_with(';') {
            }
            // Is a label
            
        }
    });
    return vec![];
}

fn second_pass(input: &[INSTR]) -> Vec<INSTR> {
    return vec![];
}

/// Assembles from INSTR
pub fn logical_assemble(instructions: &[INSTR]) -> Vec<u8> {
    instructions.iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat()
}

pub enum Operand {
    No,
    U8(u8),
    U16(u16),
    Label(String),
}

/// Struct for simple NES program debugging.
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
                Label(_) => {
                    panic!("labels have to be resolved");
                },
            }
        }
        else {
            panic!("no such instruction")
        }
    }
}


