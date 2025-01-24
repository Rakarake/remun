use shared::Opcode;
use shared::AddressingMode;
use shared::Instruction;
use shared::opcode_iter;
use shared::INSTRUCTIONS;
use std::char;
use std::fmt;
use std::collections::HashMap;

pub const MORG: u32 = 3;

#[derive(Debug)]
pub struct AsmnesError {
    line: usize,
    cause: String,
}

impl fmt::Display for AsmnesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error on line: {}, cause: {}", self.line, self.cause)
    }
}

// lables: 16bit, variables: 8bit
// First pass: lex everything, get a map of labels->address and variables->values
// Second pass: codegen over lexed input resolving using map of labels/variables 

// Lexing: we should use &str!

/// Does not produce the finalized binary, only a local byte array
pub fn simple_assemble(input: &str) -> Result<Vec<u8>, AsmnesError> {
    logical_assemble(&second_pass(&first_pass(input)?)?)
}

/// Parsing state
struct State {
    line: usize,
}

fn p_opcode<'a>(i: &'a str, s: &State) -> Result<(Opcode, &'a str), AsmnesError> {
    if i.len() >= 3 {
        let word = &i[..3];
        if let Some(o) = opcode_iter().find_map(|o| if o.to_string() == word { Some(o) } else { None }) {
            Ok((o, &i[3..]))
        } else {
            Err(AsmnesError { line: s.line, cause: "not an opcode".to_string() })
        }
    } else {
        Err(AsmnesError { line: s.line, cause: "end of file".to_string() })
    }
}

fn p_optional_spacing(i: &str) -> &str {
    let mut i = i;
    while i.starts_with(' ') || i.starts_with('\t') {
        i = &i[1..];
    }
    i
}

fn p_spacing<'a>(i: &'a str, s: &State) -> Result<((), &'a str), AsmnesError> {
    if i.starts_with(' ') || i.starts_with('\t') {
        Ok(((), p_optional_spacing(i)))
    } else {
        Err(AsmnesError { line: s.line, cause: "expected spacing".to_string() })
    }
}

fn p_label<'a>(i: &'a str, s: &State) -> Result<(String, &'a str), AsmnesError> {
    
}

fn p_line<'a>(i: &'a str, s: &State) -> Result<Option<(&'a str, INSTR)>, AsmnesError> {
    // label or indented ISNTR
    match p_spacing(i, s) {
        Ok(((), i)) => {
        },
        Err(_) => {
            // label
        },
    } p_spacing(i, s);
    Err(AsmnesError { line: s.line, cause: "end of file".to_string() })
}

/// Lexing
fn first_pass(input: &str) -> Result<Vec<INSTR>, AsmnesError> {
    // try parse an opcode
    if input.len() >= 3 {
        let glob = &input[..2];

    }
    return Ok(vec![]);
}

fn second_pass(input: &[INSTR]) -> Result<Vec<INSTR>, AsmnesError> {
    return Ok(vec![]);
}

/// Assembles from INSTR
pub fn logical_assemble(instructions: &[INSTR]) -> Result<Vec<u8>, AsmnesError> {
    Ok(instructions.iter().map(|i| i.get_bytes()).collect::<Vec<Vec<u8>>>().concat())
}

/// Struct for simple NES program debugging.
pub struct INSTR(pub Opcode, pub AddressingMode, pub Operand);

pub enum Operand {
    No,
    U8(u8),
    U16(u16),
    Label(String),
    Variable(String),
}

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
                Variable(_) => {
                    panic!("variables have to be resolved");
                },
            }
        }
        else {
            panic!("no such instruction")
        }
    }
}


