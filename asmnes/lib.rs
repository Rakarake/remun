#![feature(let_chains)]

pub mod lexer;
pub mod parser;

use lexer::lex;
use parser::parse;
use shared::opcode_addressing_modes;
use shared::AddressingMode;
use shared::CODEPOINTS;
use shared::Codepoint;
use shared::Ines;
use shared::Opcode;
use std::collections::HashMap;
use std::fmt;
use std::fs;

/// Helper macro to return an error with context
macro_rules! err {
    ($msg:expr, $line_number:expr) => {
        AsmnesError {
            line: $line_number,
            cause: $msg.to_string(),
            asmnes_file: file!(),
            asmnes_line: line!(),
            asmnes_column: column!(),
        }
    };
}

/// Fully assemble a program.
pub fn assemble(program: &str) -> Result<Ines, AsmnesError> {
    logical_assemble(&parse(lex(program)?)?)
}

pub fn assemble_from_file(path: &str) -> Result<Ines, AsmnesError> {
    assemble(&fs::read_to_string(path).map_err(|e| err!(format!("failed to load file: {e}"), 0))?)
}

/// Disassembles as many bytes as possible, returns how many bytes were used
pub fn disassemble(data: &[u8]) -> (Vec<Instruction>, usize) {
    let mut output: Vec<Instruction> = Vec::new();
    let mut pointer = data;
    while let Some((instruction, skipped)) = Instruction::from_bytes(pointer) {
        output.push(instruction);
        pointer = &pointer[skipped..];
    }
    (output, data.len() - pointer.len())
}

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

impl std::error::Error for AsmnesError {}

impl fmt::Display for AsmnesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error on line: {},\ncause: {},\nasmnes - f: {}, l: {}, r: {}",
            self.line, self.cause, self.asmnes_file, self.asmnes_line, self.asmnes_column
        )
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

impl Instruction {
    /// Constructs Statement from 1,2 or 3 bytes, also returns
    /// how many bytes were read.
    pub fn from_bytes(bytes: &[u8]) -> Option<(Self, usize)> {
        let mut itr = bytes.iter();
        let first = *itr.next()?;
        let opcode = Opcode::from(first);
        let addressing_mode = AddressingMode::from(first);
        let arity = addressing_mode.arity();
        let operand = match arity {
            0 => {Operand::No}
            1 => {Operand::U8(*itr.next()?)}
            2 => {Operand::U16(((*itr.next()? as u16) << 8) + *itr.next()? as u16)},
            _ => panic!("internal error, impossible arity"),
        };
        Some((Instruction(opcode, addressing_mode, operand), arity + 1))
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        use AddressingMode::*;
        match self.1 {
            IMM => { write!(f, " #{}", self.2) }
            IMPL => { write!(f, "") }
            A => { write!(f, " A") }
            IND => { write!(f, " ({})", self.2) }
            IND_Y => { write!(f, " ({}), Y", self.2) }
            X_IND => { write!(f, " ({}, X)", self.2) }
            _ => { write!(f, " {}", self.2) }
        }
    }
}

/// A possible line in the assembly
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Instruction(Instruction),
    Label(String),
    Directive(Directive),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct DStatement {
    statement: Statement,
    line: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operand {
    No,
    U8(u8),
    U16(u16),
    Label(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // display numbers in hex by default
        match self {
            Operand::U8(n) => {
                write!(f, "${n:02X}")
            }
            Operand::U16(n) => {
                write!(f, "${n:04X}")
            }
            Operand::No => {
                write!(f, "")
            }
            Operand::Label(l) => {
                write!(f, "{l}")
            }
        }
    }
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
    Bank(u16),
    /// nr of 16KiB bank of PRG code.
    Inesprg(u16),
    /// nr of 8KiB bank of CHR data.
    Ineschr(u16),
    /// Which mapper to use.
    Inesmap(u16),
    /// Vertical (1)/Horizontal (0, or mapper controlled) mirroring.
    Inesmir(u16),
}

/// A decorated token.
#[derive(Debug, Clone)]
pub struct DToken {
    token: Token,
    line: usize,
}

/// A token, the result of lexing.
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

/// Writes a byte and advances address.
fn write_byte(
    banks: Option<&mut Vec<u8>>,
    bank: Option<u16>,
    inesprg: Option<u16>,
    ineschr: Option<u16>,
    address: &mut u16,
    line_number: usize,
    byte: u8,
) -> Result<(), AsmnesError> {
    let bank = bank.ok_or(err!("must specify .bank", line_number))? as isize;
    let inesprg = inesprg.ok_or(err!("must specify .inesprg", line_number))? as isize;
    let ineschr = ineschr.ok_or(err!("must specify .ineschr", line_number))? as isize;
    let banks = banks.ok_or(err!("all header info needs to be present", line_number))?;
    if bank >= inesprg * 2 + ineschr {
        return Err(err!(format!("bank {bank} does not exist"), line_number));
    }
    use std::cmp::min;
    let offset = min(bank, inesprg * 2) * 1024 * 8
        + if bank > inesprg * 2 {
            (bank - inesprg * 2) * 1024 * 8
        } else {
            0
        };
    let bank_address = (*address & 0b0001111111111111) as usize;
    *banks.get_mut(bank_address + offset as usize).ok_or(err!(
        format!("internal assembler error, should not happen!"),
        line_number
    ))? = byte;
    *address += 1;
    Ok(())
}

fn create_banks(inesprg: u16, ineschr: u16) -> Vec<u8> {
    vec![0; inesprg as usize * 1024 * 16 + ineschr as usize * 1024 * 8]
}

// TODO fix "current_bank" naming
// TODO abstract similar errors (i.g. write_byte and building the final struct)
// TODO split code into modules
/// Takes a high-level representation of the program and creates the final output
/// (hopefully).
pub fn logical_assemble(program: &[DStatement]) -> Result<Ines, AsmnesError> {
    struct UnresolvedLabel {
        bank: Option<u16>,
        address: u16,
        label: String,
        line: usize,
    }
    let mut banks: Option<Vec<u8>> = None;
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut unresolved_labels: Vec<UnresolvedLabel> = Vec::new();
    let mut mapper: Option<u16> = None;
    let mut mirroring: Option<u16> = None;
    let mut inesprg: Option<u16> = None;
    let mut ineschr: Option<u16> = None;

    let mut address: u16 = 0;
    let mut current_bank: Option<u16> = None;

    // first pass
    for DStatement { statement, line } in program {
        let line = *line;
        /// Write a byte!
        macro_rules! wb {
            ($arg:expr) => {{
                write_byte(
                    banks.as_mut(),
                    current_bank,
                    inesprg,
                    ineschr,
                    &mut address,
                    line,
                    $arg,
                )?;
            }};
        }
        match statement {
            Statement::Comment(_) => {}
            Statement::Label(l) => {
                if labels.insert(l.clone(), address).is_some() {
                    return Err(err!("label already defined", line));
                }
            }
            Statement::Directive(d) => match d {
                Directive::Bank(b) => {
                    current_bank = Some(*b);
                }
                Directive::Org(a) => {
                    address = *a;
                }
                Directive::Ds(n) => {
                    address += n;
                }
                Directive::Db(b) => {
                    wb!(*b);
                }
                Directive::Inesprg(n) => {
                    if inesprg.is_some() {
                        return Err(err!(".inesprg is already specified", line));
                    } else {
                        inesprg = Some(*n);
                        if let Some(c_n) = ineschr {
                            banks = Some(create_banks(*n, c_n));
                        }
                    }
                }
                Directive::Ineschr(n) => {
                    if ineschr.is_some() {
                        return Err(err!(".ineschr is already specified", line));
                    } else {
                        ineschr = Some(*n);
                        if let Some(p_n) = inesprg {
                            banks = Some(create_banks(p_n, *n));
                        }
                    }
                }
                Directive::Inesmap(n) => {
                    mapper = Some(*n);
                }
                Directive::Inesmir(n) => {
                    mirroring = Some(*n);
                }
            },
            Statement::Instruction(Instruction(op, a, operand)) => {
                if let Some(index) = CODEPOINTS.iter().position(
                    |Codepoint {
                         opcode,
                         addressing_mode,
                     }| { op == opcode && a == addressing_mode },
                ) {
                    wb!(index as u8);
                    let byte_len = match operand {
                        Operand::No => 1,
                        Operand::U8(b) => {
                            wb!(*b);
                            2
                        }
                        Operand::U16(bs) => {
                            let [lo, hi] = bs.to_le_bytes();
                            wb!(lo);
                            wb!(hi);
                            3
                        }
                        Operand::Label(l) => {
                            unresolved_labels.push(UnresolvedLabel {
                                bank: current_bank,
                                address,
                                label: l.clone(),
                                line,
                            });
                            // skip over bytes in the meantime
                            address += 2;
                            3
                        }
                    };
                    if byte_len != a.get_len() {
                        return Err(err!(
                            format!(
                                "instruction expected argument of {} bytes but got {} bytes",
                                a.get_len() - 1,
                                byte_len - 1
                            ),
                            line
                        ));
                    }
                }
            }
        }
    }

    // second pass: go over all unresolved labels and fill them in
    for UnresolvedLabel {
        bank,
        mut address,
        label,
        line,
    } in unresolved_labels
    {
        let value = labels.get(&label).ok_or(err!("label not declared!", 0))?;
        let [lo, hi] = value.to_le_bytes();
        write_byte(
            banks.as_mut(),
            bank,
            inesprg,
            ineschr,
            &mut address,
            line,
            lo,
        )?;
        write_byte(
            banks.as_mut(),
            bank,
            inesprg,
            ineschr,
            &mut address,
            line,
            hi,
        )?;
    }

    Ok(Ines {
        inesprg: inesprg.ok_or(err!("need to specify .inesprg", 0))?,
        ineschr: ineschr.ok_or(err!("need to specify .ineschr", 0))?,
        mirroring: mirroring.ok_or(err!("need to specify .inesmir", 0))?,
        mapper: mapper.ok_or(err!("need to specify .inesmap", 0))?,
        banks: banks.ok_or(err!("all header information needs to be specified", 0))?,
        labels,
    })
}
