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
/// a INES file. Does not contain banks.
pub struct AsmnesOutput {
    pub banks: Vec<Vec<u8>>,
    pub labels: HashMap<String, u16>,
}

/// Helper macro to return an error with context
macro_rules! err {
    ($msg:expr, $line_number:expr) => {
        AsmnesError {
            line: $line_number,
            cause: $msg.to_string(),
            asmnes_file: file!(),
            asmnes_line: line!(),
            asmnes_column: column!()
        }
    };
}

/// Writes a byte and advances address.
fn write_byte(banks: &mut Vec<Vec<u8>>, bank: Option<usize>, address: &mut u16, line_number: usize, byte: u8) -> Result<(), AsmnesError> {
    if let Some(b) = bank {
        // get the bank
        let bank: &mut Vec<u8> = banks.get_mut(b).ok_or(err!(format!("bank {b} does not exist"), line_number))?;
        // write to bank
        *bank.get_mut(*address as usize).ok_or(err!(format!("address {address} is outside of bank {b}'s range"), line_number))? = byte;
        *address += 1;
        Ok(())
    } else {
        Err(err!("must specify bank", line_number))
    }
}

fn logical_assemble(program: &[Line], mut banks: Vec<Vec<u8>>) -> Result<AsmnesOutput, AsmnesError> {
    struct UnresolvedLabel {
        bank: Option<usize>,
        address: u16,
        label: String,
        line_number: usize,
    }

    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut unresolved_labels: Vec<UnresolvedLabel> = Vec::new();
    
    // first pass
    {
        let mut address: u16 = 0;
        let mut line_number: usize = 1;
        let mut current_bank: Option<usize> = None;

        for line in program {
            match line {
                Line::Comment(_) => {},
                Line::Label(l) => {
                    if labels.insert(l.clone(), address).is_some() {
                        return Err(err!("label already defined", line_number));
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
                        Directive::Org(a) => {
                            address = *a;
                        },
                        Directive::Ds(n) => {
                            address += n;
                        },
                        Directive::Db(b) => {
                            write_byte(&mut banks, current_bank, &mut address, line_number, *b)?;
                        },
                    }
                },
                Line::Instruction(Instruction(op, a, operand)) => {
                    if let Some(index) = 
                        CODEPOINTS.iter().position(|Codepoint { opcode, addressing_mode }| {
                            op == opcode && a == addressing_mode
                        })
                    {
                        write_byte(&mut banks, current_bank, &mut address, line_number, index as u8)?;
                        let byte_len = match operand {
                            Operand::No => 1,
                            Operand::U8(b) => {
                                write_byte(&mut banks, current_bank, &mut address, line_number, *b)?;
                                2
                            },
                            Operand::U16(bs) => {
                                let [lo, hi] = bs.to_le_bytes();
                                write_byte(&mut banks, current_bank, &mut address, line_number, lo)?;
                                write_byte(&mut banks, current_bank, &mut address, line_number, hi)?;
                                3
                            },
                            Operand::Label(l) => {
                                unresolved_labels.push(UnresolvedLabel { bank: current_bank, address, label: l.clone(), line_number });
                                // skip over bytes in the meantime
                                address += 2;
                                3
                            },
                        };
                        if byte_len != a.get_len() {
                            return Err(err!(
                                format!(
                                    "instruction expected argument of {} bytes but got {} bytes",
                                    a.get_len()-1, byte_len-1
                                ),
                            line_number));
                        }
                    }
                },
            }
            line_number += 1;
        }
    }
    
    // second pass: go over all unresolved labels and fill them in
    for UnresolvedLabel { bank, mut address, label, line_number } in unresolved_labels {
        let value = labels.get(&label).ok_or(err!("label not declared!", 0))?;
        let [lo, hi] = value.to_le_bytes();
        write_byte(&mut banks, bank, &mut address, line_number, lo)?;
        write_byte(&mut banks, bank, &mut address, line_number, hi)?;
    }
    
    Ok(AsmnesOutput { banks, labels })
}

