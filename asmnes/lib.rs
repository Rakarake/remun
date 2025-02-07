#![feature(let_chains)]

/// For now, only logical assemble

use shared::Opcode;
use shared::AddressingMode;
use shared::Codepoint;
use shared::CODEPOINTS;
use std::fmt;
use std::collections::HashMap;

// TODO make a trhow! macro that prints the assembler line/column, and automates creating the error
#[derive(Debug)]
pub struct AsmnesError {
    line: usize,
    cause: String,
    /// The assembler file
    asmnes_file: &'static str,
    /// The assembler line
    asmnes_line: u32,
    /// The assembler column
    asmnes_column: u32,
}

impl fmt::Display for AsmnesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error on line: {}, cause: {}", self.line, self.cause)
    }
}

/// An instruction, nothing else
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Instruction(pub Opcode, pub AddressingMode, pub Operand);

/// A possible line in the assembly
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line {
    Instruction(Instruction),
    Label(String),
    Directive(Directive),
    Comment(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operand {
    No,
    U8(u8),
    U16(u16),
    Label(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Directive {
    /// Reserve n bytes
    Ds(u16),
    Db(u8),
    Org(u16),
    Bank(usize),
}

/// The output of a logical assembly, should contain everything to create
/// a INES file.
pub struct AsmnesOutput {
    /// Banks and their content
    pub program: Vec<Vec<u8>>,
    pub labels: HashMap<String, u16>,
}

/// Helper macro to return an error with context
macro_rules! err {
    ($msg:expr, $line_number:expr) => {
        Err(AsmnesError {
            line: $line_number,
            cause: $msg.to_string(),
            asmnes_file: file!(),
            asmnes_line: line!(),
            asmnes_column: column!()
        })
    };
}

fn logical_assemble(program: &[Line], banks: &mut Vec<Vec<u8>>) -> Result<AsmnesOutput, AsmnesError> {
    let mut current_bank: Option<usize> = None;
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut line_number: usize = 1;
    let mut address: u16 = 0;
    let mut unresolved_labels: Vec<(u16, String)> = Vec::new();
    
    for line in program {
        match line {
            Line::Comment(_) => {},
            Line::Label(l) => {
                if labels.insert(l.clone(), address).is_some() {
                    return err!("label already defined", line_number);
                }
            },
            Line::Directive(d) => {
                match d {
                    Directive::Bank(b) => {
                        if *b >= banks.len() {
                            // add missing banks
                            banks.append(&mut vec![Vec::new(); b+1 - banks.len()]);
                            current_bank = Some(*b);
                        }
                    },
                }
            },
            Line::Instruction(Instruction(op, a, operand)) => {
                if let Some(bank) = current_bank {
                    if let Some(index) = 
                        CODEPOINTS.iter().position(|Codepoint { opcode, addressing_mode }| {
                            op == opcode && a == addressing_mode
                        })
                    {
                        banks[bank][address as usize] = index as u8;
                        address += 1;
                        let byte_len = match operand {
                            Operand::No => 1,
                            // TODO make sure that the operand type matches the addressing mode,
                            // first fix a throw macro,
                            // will otherwise create really nasty bugs.
                            Operand::U8(b) => {
                                banks[bank][address as usize] = *b;
                                address += 1;
                                2
                            },
                            Operand::U16(bs) => {
                                let [lo, hi] = bs.to_le_bytes();
                                banks[bank][address as usize] = lo;
                                banks[bank][address as usize + 1] = hi;
                                address += 2;
                                3
                            },
                            Operand::Label(l) => {
                                unresolved_labels.push((address, l.clone()));
                                // skip over bytes in the meantime
                                address += 2;
                                3
                            },
                        };
                        if byte_len != a.get_len() {
                            return err!(format!(
                                "instruction expected argument of {} bytes but got {} bytes",
                                a.get_len()-1, byte_len-1
                            ), line_number);
                        }
                    }
                } else {
                    return err!("must specify bank", line_number);
                }
            },
        }
        line_number += 1;
    }
    
    Ok(AsmnesOutput { program: banks, labels })
}

///// Assembles from INSTR
//fn logical_assemble_second_pass(instructions: &[INSTRD]) -> Result<Vec<u8>, AsmnesError> {
//    let mut bytes: Vec<u8> = Vec::new();
//    for i in instructions {
//        bytes.append(&mut i.get_bytes()?);
//    }
//    Ok(bytes)
//}
//
///// Accepts labels and instructions that can use labels
//pub fn logical_assemble(program: &[INSTRL]) -> Result<AsmnesOutput, AsmnesError> {
//    // Does not have labels filled out after this step
//    let mut labels: HashMap<String, u16> = HashMap::new();
//    let mut instrs: Vec<INSTRD> = Vec::new();
//    let mut address: u16 = 0;
//    let mut line: usize = 1;
//    for instrl in program {
//        match instrl {
//            INSTRL::INSTR(i) => {
//                instrs.push(INSTRD { instr: INSTRL::INSTR(i.clone()), line, address });
//                address += i.1.get_len();
//            },
//            INSTRL::LABEL(l) => {
//                instrs.push(INSTRD { instr: INSTRL::LABEL(l.clone()), line, address });
//                if labels.insert(l.clone(), address).is_some() {
//                    return Err(AsmnesError { line, cause: "label already defined".to_string() });
//                }
//            },
//            INSTRL::DIR(d) => {
//                instrs.push(INSTRD { instr: INSTRL::DIR(d.clone()), line, address });
//                address += d.get_len();
//                match d {
//                    Directive::ORG(addr) => {
//                        address = *addr;
//                    },
//                    _ => {},
//                }
//            },
//        }
//        line += 1;
//    }
//    for INSTRD { instr, line: _, address: _ } in &mut instrs {
//        if let INSTRL::INSTR(i) = instr && let Operand::Label(l) = &mut i.2 {
//            if let Some(address) = labels.get(&*l) {
//                i.2 = Operand::U16(*address);
//            } else {
//                return Err(AsmnesError { line: 0, cause: "label does not exist".to_string() });
//            }
//        }
//    }
//    
//    Ok(AsmnesOutput { program: logical_assemble_second_pass(&instrs)?, labels })
//}
//
//impl INSTRD {
//    pub fn get_bytes(&self) -> Result<Vec<u8>, AsmnesError> {
//        let INSTRD { instr, line, address: _ }  = self;
//        match instr {
//            INSTRL::INSTR(i) => {
//                let INSTR(op, a, operand) = i;
//                if let Some(index) = 
//                    INSTRUCTIONS.iter().position(|Instruction { opcode, addressing_mode }| {
//                        op == opcode && a == addressing_mode
//                    })
//                {
//                    use Operand::*;
//                    match operand {
//                        No => Ok(vec![index as u8]),
//                        U8(b) => Ok(vec![index as u8, *b]),
//                        U16(bs) => {
//                            let mut x = vec![index as u8];
//                            x.extend_from_slice(&bs.to_le_bytes());
//                            Ok(x)
//                        },
//                        Label(_) => {
//                            Err(AsmnesError { line: *line, cause: "labels have to be resolved!".to_string() })
//                        },
//                        Variable(_) => {
//                            Err(AsmnesError { line: *line, cause: "variables have to be resloved!".to_string() })
//                        },
//                    }
//                }
//                else {
//                    Err(AsmnesError { line: *line, cause: "no such opcode-operand combination".to_string() })
//                }
//            },
//            // labels don't take up any space 🤷
//            INSTRL::LABEL(_) => { Ok(Vec::new()) },
//            INSTRL::DIR(d) => { Ok(d.get_bytes()) },
//        }
//    }
//}
//
//impl Directive {
//    fn get_bytes(&self) -> Vec<u8> {
//        match self {
//            Directive::DB(n) => {
//                vec![0; *n as usize]
//            },
//            _ => vec![],
//        }
//    }
//    fn get_len(&self) -> u16 {
//        match self {
//            Directive::DB(_) => {
//                1
//            },
//            Directive::DS(n) => {
//                // TODO weird, length but no bytes
//                *n
//            },
//            _ => 0,
//        }
//    }
//}




