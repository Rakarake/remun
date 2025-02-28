#![feature(let_chains)]

/// For now, only logical assemble

use shared::Opcode;
use shared::AddressingMode;
use shared::Codepoint;
use shared::CODEPOINTS;
use shared::Range;
use std::fmt;
use std::collections::HashMap;

// TODO make a trhow! macro that prints the assembler line/column, and automates creating the error
//#[derive(Debug)]
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
        write!(f, "error on line: {},\ncause: {},\nasmnes - f: {}, l: {}, r: {}",
            self.line, self.cause, self.asmnes_file, self.asmnes_line, self.asmnes_column)
    }
}

impl fmt::Debug for AsmnesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
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
    /// nr of 16KB bank of PRG code
    Inesprg(usize), 
    /// nr of 8KB bank of CHR data
    Ineschr(usize), 
}

/// The output of a logical assembly, should contain everything to create
/// a INES file. Does not contain banks.
pub struct AsmnesOutput {
    pub banks: Vec<Bank>,
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

#[derive(Clone)]
pub struct Bank {
    pub data: Vec<u8>,
}

/// Writes a byte and advances address.
fn write_byte(banks: &mut [Bank], bank: Option<usize>, address: &mut u16, line_number: usize, byte: u8) -> Result<(), AsmnesError> {
    if let Some(b) = bank {
        // get the bank
        let bank: &mut Bank = banks.get_mut(b).ok_or(err!(format!("bank {b} does not exist"), line_number))?;
        // write to bank
        *bank.data.get_mut((*address & 0b0001111111111111) as usize).
            ok_or(err!(format!("address {address} is outside of bank {b}'s range"), line_number))? = byte;
        *address += 1;
        // TODO maybe add overflow check?
        Ok(())
    } else {
        Err(err!("must specify bank", line_number))
    }
}

#[derive(Debug)]
pub struct DToken { token: Token, line: usize }
#[derive(Debug)]
pub enum Token {
    Ident(String),
    Directive(String),
    Num(u16),
    ParenOpen,
    ParenClose,
    Comma,
    Hash,
    Colon,
    Newline,
}

#[derive(PartialEq, Clone, Copy)]
enum LexState {
    Awaiting,
    ReadingBin,
    ReadingHex,
    ReadingDec,
    ReadingIdent,
    ReadingDirective,
    ReadingComment,
}

fn get_radix(ls: LexState, line_number: usize) -> Result<u32, AsmnesError> {
    match ls {
        LexState::ReadingHex => Ok(16),
        LexState::ReadingBin => Ok(2),
        LexState::ReadingDec => Ok(10),
        _ => Err(err!("internal parsing error when getting radix", line_number)),
    }
}

// A delimiter ends the previous work
fn delimiter(state: &mut LexState, line: usize, output: &mut Vec<DToken>, acc: &mut String) -> Result<(), AsmnesError> {
    match state {
        LexState::ReadingIdent => {
            output.push(DToken { token: Token::Ident(acc.clone()), line });
            *state = LexState::Awaiting;
        },
        LexState::ReadingHex | LexState::ReadingBin | LexState::ReadingDec => {
            output.push(DToken {
                token: Token::Num(u16::from_str_radix(acc, get_radix(*state, line)?)
                           .map_err(|e| err!(format!("number parse error: {}", e), line))?),
                line,
            });
            *state = LexState::Awaiting;
        },
        LexState::ReadingDirective => {
            output.push(DToken { token: Token::Directive(acc.clone()), line });
            *state = LexState::Awaiting;
        },
        LexState::Awaiting  => {
            *state = LexState::Awaiting;
        },
        LexState::ReadingComment => {},
    }
    acc.clear();
    Ok(())
}

pub fn lex(program: &str) -> Result<Vec<DToken>, AsmnesError> {
    let mut output: Vec<DToken> = Vec::new();
    let mut line: usize = 1;
    let mut state: LexState = LexState::Awaiting;
    let mut acc: String = String::new();
    for c in program.chars() {
        match c {
            '\n' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::Newline, line });
                line += 1;
                // After commnet, start reading again
                state = LexState::Awaiting;
            },
            '.' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                state = LexState::ReadingDirective;
            },
            '(' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::ParenOpen, line });
            },
            ')' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::ParenClose, line });
            },
            ',' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::Comma, line });
            },
            '#' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::Hash, line });
            },
            ':' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken { token: Token::Colon, line });
            },
            ' ' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
            },
            '\t' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
            },
            ';' => state = LexState::ReadingComment,
            '$' => state = LexState::ReadingHex,
            '%' => state = LexState::ReadingBin,
            _ => {
                match state {
                    LexState::Awaiting => {
                        // Start reading ident or decimal number
                        if c.is_numeric() {
                            acc.push(c);
                            state = LexState::ReadingDec;
                        } else if c.is_alphabetic() {
                            acc.push(c);
                            state = LexState::ReadingIdent;
                        } else {
                            return Err(err!("parse error", line))
                        }
                    },
                    LexState::ReadingComment => { },
                    _ => acc.push(c),
                }
            },
        }
    }
    Ok(output)
}

fn parse(program: Vec<DToken>) -> Result<Vec<Line>, AsmnesError> {
    let mut itr = program.iter();
    while let Some(DToken { token, line }) = itr.next() {
        match token {
            Token::Directive(d) => {
                // TODO need some table of directive names to check for
                // might want to change to all-capital letters, then check both
                // capital/noncapital versions, should be done for opcodes as well
            },
            Token::Ident(i) => {
                // label or opcode?
            },
            Token::Newline => {
            },
            _ => {
                return Err(err!("parse error", *line));
            },
        }
    }
    Ok(vec![])
}

pub fn logical_assemble(program: &[Line]) -> Result<AsmnesOutput, AsmnesError> {
    struct UnresolvedLabel {
        bank: Option<usize>,
        address: u16,
        label: String,
        line_number: usize,
    }
    let mut banks: Vec<Bank> = Vec::new();
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
                            if *b < banks.len() {
                                current_bank = Some(*b);
                            } else {
                                return Err(err!(format!("trying to define bank {b} but only banks 0 to {} exist", banks.len()), line_number));
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
                        Directive::Inesprg(n) => {
                            for _ in 0..*n {
                                banks.push(Bank {
                                    data: vec![0; 4000],
                                });
                            }
                        },
                        Directive::Ineschr(n) => {
                            // TODO make a difference between chr and prg banks?
                            for _ in 0..*n {
                                banks.push(Bank {
                                    data: vec![0; 4000],
                                });
                            }
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

