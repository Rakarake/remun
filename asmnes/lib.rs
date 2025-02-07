#![feature(let_chains)]

/// For now, only logical assemble

use shared::Opcode;
use shared::AddressingMode;
use shared::Instruction;
use shared::INSTRUCTIONS;
use std::fmt;
use std::collections::HashMap;

pub struct AsmnesOutput {
    pub program: Vec<u8>,
    pub labels: HashMap<String, u16>,
}

/// An instruction, nothing else
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct INSTR(pub Opcode, pub AddressingMode, pub Operand);

/// A possible line in the assembly
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum INSTRL {
    INSTR(INSTR),
    LABEL(String),
    DIR(Directive),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operand {
    No,
    U8(u8),
    U16(u16),
    Label(String),
    Variable(String),
}

/// INSTR, decorated with metadata
struct INSTRD {
    instr: INSTRL,
    line: usize,
    address: u16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Directive {
    /// Reserve n bytes
    DS(u16),
    DB(u8),
    ORG(u16),
}

/// Assembles from INSTR
fn logical_assemble_second_pass(instructions: &[INSTRD]) -> Result<Vec<u8>, AsmnesError> {
    let mut bytes: Vec<u8> = Vec::new();
    for i in instructions {
        bytes.append(&mut i.get_bytes()?);
    }
    Ok(bytes)
}

/// Accepts labels and instructions that can use labels
pub fn logical_assemble(program: &[INSTRL]) -> Result<AsmnesOutput, AsmnesError> {
    // Does not have labels filled out after this step
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut instrs: Vec<INSTRD> = Vec::new();
    let mut address: u16 = 0;
    let mut line: usize = 1;
    for instrl in program {
        match instrl {
            INSTRL::INSTR(i) => {
                instrs.push(INSTRD { instr: INSTRL::INSTR(i.clone()), line, address });
                address += i.1.get_len();
            },
            INSTRL::LABEL(l) => {
                instrs.push(INSTRD { instr: INSTRL::LABEL(l.clone()), line, address });
                if labels.insert(l.clone(), address).is_some() {
                    return Err(AsmnesError { line, cause: "label already defined".to_string() });
                }
            },
            INSTRL::DIR(d) => {
                instrs.push(INSTRD { instr: INSTRL::DIR(d.clone()), line, address });
                address += d.get_len();
                match d {
                    Directive::ORG(addr) => {
                        address = *addr;
                    },
                    _ => {},
                }
            },
        }
        line += 1;
    }
    for INSTRD { instr, line: _, address: _ } in &mut instrs {
        if let INSTRL::INSTR(i) = instr && let Operand::Label(l) = &mut i.2 {
            if let Some(address) = labels.get(&*l) {
                i.2 = Operand::U16(*address);
            } else {
                return Err(AsmnesError { line: 0, cause: "label does not exist".to_string() });
            }
        }
    }
    
    Ok(AsmnesOutput { program: logical_assemble_second_pass(&instrs)?, labels })
}

impl INSTRD {
    pub fn get_bytes(&self) -> Result<Vec<u8>, AsmnesError> {
        let INSTRD { instr, line, address: _ }  = self;
        match instr {
            INSTRL::INSTR(i) => {
                let INSTR(op, a, operand) = i;
                if let Some(index) = 
                    INSTRUCTIONS.iter().position(|Instruction { opcode, addressing_mode }| {
                        op == opcode && a == addressing_mode
                    })
                {
                    use Operand::*;
                    match operand {
                        No => Ok(vec![index as u8]),
                        U8(b) => Ok(vec![index as u8, *b]),
                        U16(bs) => {
                            let mut x = vec![index as u8];
                            x.extend_from_slice(&bs.to_le_bytes());
                            Ok(x)
                        },
                        Label(_) => {
                            Err(AsmnesError { line: *line, cause: "labels have to be resolved!".to_string() })
                        },
                        Variable(_) => {
                            Err(AsmnesError { line: *line, cause: "variables have to be resloved!".to_string() })
                        },
                    }
                }
                else {
                    Err(AsmnesError { line: *line, cause: "no such opcode-operand combination".to_string() })
                }
            },
            // labels don't take up any space 🤷
            INSTRL::LABEL(_) => { Ok(Vec::new()) },
            INSTRL::DIR(d) => { Ok(d.get_bytes()) },
        }
    }
}

impl Directive {
    fn get_bytes(&self) -> Vec<u8> {
        match self {
            Directive::DB(n) => {
                vec![0; *n as usize]
            },
            _ => vec![],
        }
    }
    fn get_len(&self) -> u16 {
        match self {
            Directive::DB(_) => {
                1
            },
            Directive::DS(n) => {
                // TODO weird, length but no bytes
                *n
            },
            _ => 0,
        }
    }
}


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


