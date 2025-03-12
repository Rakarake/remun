#![feature(let_chains)]

/// For now, only logical assemble

use shared::Opcode;
use shared::AddressingMode;
use shared::Codepoint;
use shared::CODEPOINTS;
use shared::Range;
use shared::opcode_addressing_modes;
use std::fmt;
use std::collections::HashMap;
use std::fs;

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
    /// Reserve n bytes.
    Ds(u16),
    /// Manually put bytes into memory.
    Db(u8),
    /// Sets the memory location where new labels/instructions are set, the
    /// 13nth lower bits determines the offset into bank memroy.
    Org(u16),
    /// Switches to bank.
    Bank(usize),
    /// nr of 16KiB bank of PRG code.
    Inesprg(usize), 
    /// nr of 8KiB bank of CHR data.
    Ineschr(usize), 
    /// Which mapper to use.
    Inesmap(usize),
    /// Vertical (1)/Horizontal (0, or mapper controlled) mirroring.
    Inesmir(usize),
}

/// The output of a logical assembly, should contain everything to create
/// a INES file.
pub struct AsmnesOutput {
    /// nr of 16KiB bank of PRG code.
    pub prg_rom_size: usize,
    /// nr of 8KiB bank of CHR code.
    pub chr_rom_size: usize,
    pub mirroring: usize,
    /// The iNES mapper index, does not fully describe the hardware
    pub mapper: usize,
    pub banks: Vec<Bank>,
    /// Debug information
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

#[derive(Debug, Clone)]
pub struct DToken { token: Token, line: usize }
#[derive(Debug, Clone, PartialEq)]
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
    X,
    Y,
    A,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    println!("state: {:?}", ls);
    match ls {
        LexState::ReadingHex => Ok(16),
        LexState::ReadingBin => Ok(2),
        LexState::ReadingDec => Ok(10),
        _ => Err(err!("internal parsing error when getting radix", line_number)),
    }
}

/// Fully assemble a program.
pub fn assemble(program: &str) -> Result<AsmnesOutput, AsmnesError> {
    logical_assemble(&parse(lex(program)?)?)
}

pub fn assemble_from_file(path: &str) -> Result<AsmnesOutput, AsmnesError> {
    assemble(&fs::read_to_string(path).map_err(|e| err!(format!("failed to load file: {e}"), 0))?)
}

/// A delimiter ends the previous work, sets state to awaiting
fn delimiter(state: &mut LexState, line: usize, output: &mut Vec<DToken>, acc: &mut String) -> Result<(), AsmnesError> {
    match state {
        LexState::ReadingIdent => {
            // X & Y tokens take precedence
            if acc == "X" {
                output.push(DToken { token: Token::X, line });
            } else if acc == "Y" {
                output.push(DToken { token: Token::Y, line });
            } else if acc == "A" {
                output.push(DToken { token: Token::A, line });
            } else {
                output.push(DToken { token: Token::Ident(acc.clone()), line });
            }
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

fn parse_opcode(i: &str, line: usize) -> Result<Opcode, AsmnesError> {
    i.parse::<Opcode>().map_err(|_| err!("expected opcode", line))
}

/// Numbers after the lex stage are u16, make sure n is a u8 or throw
/// an error.
fn parse_u8(n: u16, line: usize) -> Result<u8, AsmnesError> {
    u8::try_from(n).map_err(|_| err!("expected u8", line))
}

/// Extracts the number from the dtoken.
fn parse_num(dtoken: &DToken) -> Result<u16, AsmnesError> {
    let DToken { token, line } = dtoken;
    match token {
        Token::Num(n) => {
            Ok(*n)
        }
        _ => Err(err!("expected number", *line)),
    }
}

/// Make sure this dtoken contains this token.
fn parse_expect(dtoken: &DToken, t: Token) -> Result<(), AsmnesError> {
    let DToken { token, line } = dtoken;
    if t == *token {
        Ok(())
    } else {
        Err(err!(format!("expected '{:?}', got '{:?}'", t, token), *line))
    }
}

// Helper for when you expect a new symbol
fn parse_next(i: Option<&DToken>, line: usize) -> Result<&DToken, AsmnesError> {
    i.ok_or(err!("unexpected end of line", line))
}

/// Parses lex output.
/// Note: parser is very stupid, will make assumptions that will be caught by the
/// logical assembler.
pub fn parse(program: Vec<DToken>) -> Result<Vec<Line>, AsmnesError> {
    let mut output: Vec<Line> = Vec::new();
    let mut itr = program.iter().peekable();
    let mut already_found_newline = false;
    while let Some(DToken { token, line }) = itr.next() {
        match token {
            Token::Directive(d) => {
                // TODO need some table of directive names to check for
                // might want to change to all-capital letters, then check both
                // capital/noncapital versions, should be done for opcodes as well
                match d.as_str() {
                    "org" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Org(n)));
                    },
                    "bank" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Bank(n as usize)));
                    },
                    "inesprg" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Inesprg(n as usize)));
                    },
                    "ineschr" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Ineschr(n as usize)));
                    },
                    "inesmap" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Inesmap(n as usize)));
                    },
                    "inesmir" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Inesmir(n as usize)));
                    },
                    "db" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Db(parse_u8(n, *line)?)));
                    },
                    "ds" => {
                        let t = parse_next(itr.next(), *line)?;
                        let n = parse_num(t)?;
                        output.push(Line::Directive(Directive::Ds(n)));
                    },
                    s => return Err(err!(format!("no such directive: '{}'", s), *line)),
                }
            },
            Token::Ident(i) => {
                if let Some(DToken { token, line }) = itr.next() {
                    match token {
                        // Label
                        Token::Colon => {
                            output.push(Line::Label(i.clone()));
                        },
                        // Immediate mode
                        Token::Hash => {
                            let o = parse_opcode(i, *line)?;
                            let t = parse_next(itr.next(), *line)?;
                            let n = parse_num(t)?;
                            let n = parse_u8(n, *line)?;
                            output.push(Line::Instruction(Instruction(o, AddressingMode::IMM, Operand::U8(n))));
                        }
                        Token::Num(n) => {
                            // Absolute + X/Y indexed, relative or Zero-page + X/Y indexed
                            // TODO need to know the available addressing modes for an opcode, to
                            // see if it can use zero-page, or if it's using relative
                            let o = parse_opcode(i, *line)?;
                            use AddressingMode::*;
                            if opcode_addressing_modes(&o).iter().any(|a| *a == REL) {
                                // Relative addr-mode takes precedence
                                let n = parse_u8(*n, *line)?;
                                output.push(Line::Instruction(Instruction(o, AddressingMode::REL, Operand::U8(n))));
                            } else {
                                // Check if we can use ZPG
                                let (operand, zpg) = if let Ok(operand) = parse_u8(*n, *line) && opcode_addressing_modes(&o).iter().any(|a| *a == ZPG || *a == ZPG_X || *a == ZPG_Y) {
                                    (Operand::U8(operand), true)
                                } else {
                                    (Operand::U16(*n), false)
                                };
                                // Zero page
                                if let Some(DToken { token, line }) = itr.next() {
                                    match token {
                                        Token::Newline => {
                                            // non-x/y addressed
                                            already_found_newline = true; 
                                            output.push(Line::Instruction(Instruction(o, if zpg {AddressingMode::ZPG} else {AddressingMode::ABS}, operand)));
                                            continue;
                                        },
                                        Token::Comma => {
                                            // x/y addressed
                                            let DToken { token, line } = parse_next(itr.next(), *line)?;
                                            output.push(Line::Instruction(Instruction(o, 
                                                    match token {
                                                        Token::X => if zpg {AddressingMode::ZPG_X} else {AddressingMode::ABS_X},
                                                        Token::Y => if zpg {AddressingMode::ZPG_Y} else {AddressingMode::ABS_Y},
                                                        _ => return Err(err!("expected either X or Y", *line))
                                                    },
                                            operand)));
                                        },
                                        _ => return Err(err!("expected comma", *line)),
                                    }
                                } else {
                                    // non-x/y addressed
                                    output.push(Line::Instruction(Instruction(o, if zpg {AddressingMode::ZPG} else {AddressingMode::ABS}, operand)));
                                }
                            }
                        },
                        Token::ParenOpen => {
                            // Any indirect addr-mode
                            let o = parse_opcode(i, *line)?;
                            let t = parse_next(itr.next(), *line)?;
                            let n = parse_num(t)?;
                            let DToken { token, line } = parse_next(itr.next(), *line)?;
                            match token {
                                Token::ParenClose => {
                                    // indirect or y indexed
                                    if let Ok(DToken { token, line }) = parse_next(itr.next(), *line) {
                                        match token {
                                            Token::Comma => {
                                                parse_expect(parse_next(itr.next(), *line)?, Token::Y)?;
                                                output.push(Line::Instruction(Instruction(o, AddressingMode::IND_Y, Operand::U8(parse_u8(n, *line)?))));
                                            },
                                            Token::Newline => {
                                                // indirect
                                                output.push(Line::Instruction(Instruction(o, AddressingMode::IND, Operand::U16(n))));
                                            },
                                            _ => return Err(err!("unexpected symbol when trying to parse indirect/Y-indexed addressing", *line)),
                                        }
                                    } else {
                                        //  indirect
                                        output.push(Line::Instruction(Instruction(o, AddressingMode::IND, Operand::U16(n))));
                                    }
                                },
                                Token::Comma => {
                                    // indirect x addressed
                                    output.push(Line::Instruction(Instruction(o, AddressingMode::X_IND, Operand::U8(parse_u8(n, *line)?))));
                                    parse_expect(parse_next(itr.next(), *line)?, Token::X)?;
                                    parse_expect(parse_next(itr.next(), *line)?, Token::ParenClose)?;
                                },
                                t => return Err(err!(format!("unexpected token '{:?}' when trying to parse any of the indirect addressing modes", t), *line)),
                            }
                        },
                        Token::A => {
                            // Accumulator addr-mode
                            let o = parse_opcode(i, *line)?;
                            output.push(Line::Instruction(Instruction(o, AddressingMode::A, Operand::No)));
                        },
                        Token::Newline => {
                            // Implied
                            already_found_newline = true;
                            let o = parse_opcode(i, *line)?;
                            output.push(Line::Instruction(Instruction(o, AddressingMode::IMPL, Operand::No)));
                        },
                        _ => {
                            return Err(err!("wrong token after ident", *line))
                        },
                    }
                } else {
                    let o = parse_opcode(i, *line)?;
                    output.push(Line::Instruction(Instruction(o, AddressingMode::IMPL, Operand::No)));
                }
            },
            Token::Newline => {
                already_found_newline = true;
            },
            t => {
                return Err(err!(format!("expected either an instruction, a directive or a label, got token '{:?}'", t), *line));
            },
        }
        // expecting newline after line contents
        if !already_found_newline {
            if let Some(DToken { token, line }) = itr.next() {
                if *token != Token::Newline {
                    return Err(err!("expected newline", *line));
                }
            }
        }
    }
    Ok(output)
}

/// Takes a high-level representation of the program and creates the final output
/// (hopefully).
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
    let mut mapper: Option<usize> = None;
    let mut mirroring: Option<usize> = None;
    let mut prg_banks: Option<usize> = None;
    let mut chr_banks: Option<usize> = None;
    
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
                            if banks.is_empty() {
                                prg_banks = Some(*n);
                                for _ in 0..*n {
                                    banks.push(Bank {
                                        data: vec![0; 4000],
                                    });
                                }
                            } else {
                                return Err(err!("have already specified banks!", line_number));
                            }
                        },
                        Directive::Ineschr(n) => {
                            if !banks.is_empty() {
                                chr_banks = Some(*n);
                                for _ in 0..*n {
                                    banks.push(Bank {
                                        data: vec![0; 4000],
                                    });
                                }
                            } else {
                                return Err(err!("have to specify .inesprg first!", line_number));
                            }
                        },
                        Directive::Inesmap(n) => {
                            mapper = Some(*n);
                        },
                        Directive::Inesmir(n) => {
                            mirroring = Some(*n);
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
    
    Ok(AsmnesOutput {
        prg_rom_size: prg_banks.ok_or(err!("need to specify .inesprg", 0))?,
        chr_rom_size: chr_banks.ok_or(err!("need to specify .ineschr", 0))?,
        mirroring: mirroring.ok_or(err!("need to specify .inesmir", 0))?,
        mapper: mapper.ok_or(err!("need to specify .inesmap", 0))?,
        banks,
        labels
    })
}

