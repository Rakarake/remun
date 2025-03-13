#![feature(let_chains)]

use shared::AddressingMode;
use shared::CODEPOINTS;
use shared::Codepoint;
use shared::Ines;
/// For now, only logical assemble
use shared::Opcode;
use shared::Range;
use shared::opcode_addressing_modes;
use std::collections::HashMap;
use std::fmt;
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

#[derive(Debug, Clone)]
pub struct DToken {
    token: Token,
    line: usize,
}
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
    match ls {
        LexState::ReadingHex => Ok(16),
        LexState::ReadingBin => Ok(2),
        LexState::ReadingDec => Ok(10),
        _ => Err(err!(
            "internal parsing error when getting radix",
            line_number
        )),
    }
}

/// Fully assemble a program.
pub fn assemble(program: &str) -> Result<Ines, AsmnesError> {
    logical_assemble(&parse(lex(program)?)?)
}

pub fn assemble_from_file(path: &str) -> Result<Ines, AsmnesError> {
    assemble(&fs::read_to_string(path).map_err(|e| err!(format!("failed to load file: {e}"), 0))?)
}

/// A delimiter ends the previous work, sets state to awaiting
fn delimiter(
    state: &mut LexState,
    line: usize,
    output: &mut Vec<DToken>,
    acc: &mut String,
) -> Result<(), AsmnesError> {
    match state {
        LexState::ReadingIdent => {
            // X & Y tokens take precedence
            if acc == "X" {
                output.push(DToken {
                    token: Token::X,
                    line,
                });
            } else if acc == "Y" {
                output.push(DToken {
                    token: Token::Y,
                    line,
                });
            } else if acc == "A" {
                output.push(DToken {
                    token: Token::A,
                    line,
                });
            } else {
                output.push(DToken {
                    token: Token::Ident(acc.clone()),
                    line,
                });
            }
            *state = LexState::Awaiting;
        }
        LexState::ReadingHex | LexState::ReadingBin | LexState::ReadingDec => {
            output.push(DToken {
                token: Token::Num(
                    u16::from_str_radix(acc, get_radix(*state, line)?)
                        .map_err(|e| err!(format!("number parse error: {}", e), line))?,
                ),
                line,
            });
            *state = LexState::Awaiting;
        }
        LexState::ReadingDirective => {
            output.push(DToken {
                token: Token::Directive(acc.clone()),
                line,
            });
            *state = LexState::Awaiting;
        }
        LexState::Awaiting => {
            *state = LexState::Awaiting;
        }
        LexState::ReadingComment => {}
    }
    acc.clear();
    Ok(())
}

pub fn lex(program: &str) -> Result<Vec<DToken>, AsmnesError> {
    let mut output: Vec<DToken> = Vec::new();
    let mut line: usize = 1;
    let mut state: LexState = LexState::Awaiting;
    let mut acc: String = String::new();
    /// Helper to reduce code.
    macro_rules! delimiter_then_push {
        ($token:expr) => {{
            delimiter(&mut state, line, &mut output, &mut acc)?;
            output.push(DToken {
                token: $token,
                line,
            });
        }};
    }
    for c in program.chars() {
        match c {
            '\n' => {
                delimiter_then_push!(Token::Newline);
                line += 1;
                // After commnet, start reading again
                state = LexState::Awaiting;
            }
            '.' => {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                state = LexState::ReadingDirective;
            }
            '(' => delimiter_then_push!(Token::ParenOpen),
            ')' => delimiter_then_push!(Token::ParenClose),
            ',' => delimiter_then_push!(Token::Comma),
            '#' => delimiter_then_push!(Token::Hash),
            ':' => delimiter_then_push!(Token::Colon),
            ' ' => delimiter(&mut state, line, &mut output, &mut acc)?,
            '\t' => delimiter(&mut state, line, &mut output, &mut acc)?,
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
                            return Err(err!("parse error", line));
                        }
                    }
                    LexState::ReadingComment => {}
                    _ => acc.push(c),
                }
            }
        }
    }
    Ok(output)
}

fn parse_opcode(i: &str, line: usize) -> Result<Opcode, AsmnesError> {
    i.parse::<Opcode>()
        .map_err(|_| err!("expected opcode", line))
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
        Token::Num(n) => Ok(*n),
        _ => Err(err!("expected number", *line)),
    }
}

/// Make sure this dtoken contains this token.
fn parse_expect(dtoken: &DToken, t: Token) -> Result<(), AsmnesError> {
    let DToken { token, line } = dtoken;
    if t == *token {
        Ok(())
    } else {
        Err(err!(
            format!("expected '{:?}', got '{:?}'", t, token),
            *line
        ))
    }
}

// Helper for when you expect a new symbol
fn parse_next(i: Option<&DToken>, line: usize) -> Result<&DToken, AsmnesError> {
    i.ok_or(err!("unexpected end of line", line))
}

/// Parses lex output.
/// Note: parser is very stupid, will make assumptions that will be caught by the
/// logical assembler.
pub fn parse(program: Vec<DToken>) -> Result<Vec<DStatement>, AsmnesError> {
    let mut output: Vec<DStatement> = Vec::new();
    let mut itr = program.iter().peekable();
    let mut already_found_newline = false;
    while let Some(DToken { token, line }) = itr.next() {
        let line = *line;
        match token {
            Token::Directive(d) => {
                // TODO need some table of directive names to check for
                // might want to change to all-capital letters, then check both
                // capital/noncapital versions, should be done for opcodes as well

                /// Parses next token as a u16, pushes a directive
                macro_rules! directive_push_num {
                    ($statement:expr) => {{
                        let t = parse_next(itr.next(), line)?;
                        let n = parse_num(t)?;
                        output.push(DStatement {
                            statement: Statement::Directive($statement(n)),
                            line,
                        });
                    }};
                }
                match d.as_str() {
                    "org" => directive_push_num!(Directive::Org),
                    "bank" => directive_push_num!(Directive::Bank),
                    "inesprg" => directive_push_num!(Directive::Inesprg),
                    "ineschr" => directive_push_num!(Directive::Ineschr),
                    "inesmap" => directive_push_num!(Directive::Inesmap),
                    "inesmir" => directive_push_num!(Directive::Inesmir),
                    "db" => {
                        let t = parse_next(itr.next(), line)?;
                        let n = parse_num(t)?;
                        output.push(DStatement {
                            statement: Statement::Directive(Directive::Db(parse_u8(n, line)?)),
                            line,
                        });
                    }
                    "ds" => directive_push_num!(Directive::Ds),
                    s => return Err(err!(format!("no such directive: '{}'", s), line)),
                }
            }
            Token::Ident(i) => {
                macro_rules! instruction_push {
                    ($o:expr, $a:expr, $operand:expr) => {{
                        output.push(DStatement {
                            statement: Statement::Instruction(Instruction($o, $a, $operand)),
                            line,
                        });
                    }};
                }
                if let Some(DToken { token, line }) = itr.next() {
                    let line = *line;
                    // note: the line will be the same regardless of new tokens
                    match token {
                        // Label
                        Token::Colon => {
                            output.push(DStatement {
                                statement: Statement::Label(i.clone()),
                                line,
                            });
                        }
                        // Immediate mode
                        Token::Hash => {
                            let o = parse_opcode(i, line)?;
                            let t = parse_next(itr.next(), line)?;
                            let n = parse_num(t)?;
                            let n = parse_u8(n, line)?;
                            instruction_push!(o, AddressingMode::IMM, Operand::U8(n));
                        }
                        Token::Num(n) => {
                            // Absolute + X/Y indexed, relative or Zero-page + X/Y indexed
                            // TODO need to know the available addressing modes for an opcode, to
                            // see if it can use zero-page, or if it's using relative
                            let o = parse_opcode(i, line)?;
                            use AddressingMode::*;
                            if opcode_addressing_modes(&o).iter().any(|a| *a == REL) {
                                // Relative addr-mode takes precedence
                                let n = parse_u8(*n, line)?;
                                instruction_push!(o, AddressingMode::REL, Operand::U8(n));
                            } else {
                                // Check if we can use ZPG
                                let (operand, zpg) = if let Ok(operand) = parse_u8(*n, line)
                                    && opcode_addressing_modes(&o)
                                        .iter()
                                        .any(|a| *a == ZPG || *a == ZPG_X || *a == ZPG_Y)
                                {
                                    (Operand::U8(operand), true)
                                } else {
                                    (Operand::U16(*n), false)
                                };
                                // Zero page
                                if let Some(DToken { token, line }) = itr.next() {
                                    let line = *line;
                                    match token {
                                        Token::Newline => {
                                            // non-x/y addressed
                                            already_found_newline = true;
                                            instruction_push!(
                                                o,
                                                if zpg {
                                                    AddressingMode::ZPG
                                                } else {
                                                    AddressingMode::ABS
                                                },
                                                operand
                                            );
                                            continue;
                                        }
                                        Token::Comma => {
                                            // x/y addressed
                                            let DToken { token, line } =
                                                parse_next(itr.next(), line)?;
                                            let line = *line;
                                            instruction_push!(
                                                o,
                                                match token {
                                                    Token::X => {
                                                        if zpg {
                                                            AddressingMode::ZPG_X
                                                        } else {
                                                            AddressingMode::ABS_X
                                                        }
                                                    }
                                                    Token::Y => {
                                                        if zpg {
                                                            AddressingMode::ZPG_Y
                                                        } else {
                                                            AddressingMode::ABS_Y
                                                        }
                                                    }
                                                    _ => {
                                                        return Err(err!(
                                                            "expected either X or Y",
                                                            line
                                                        ));
                                                    }
                                                },
                                                operand
                                            );
                                        }
                                        _ => return Err(err!("expected comma", line)),
                                    }
                                } else {
                                    // non-x/y addressed
                                    instruction_push!(
                                        o,
                                        if zpg {
                                            AddressingMode::ZPG
                                        } else {
                                            AddressingMode::ABS
                                        },
                                        operand
                                    );
                                }
                            }
                        }
                        Token::ParenOpen => {
                            // Any indirect addr-mode
                            let o = parse_opcode(i, line)?;
                            let t = parse_next(itr.next(), line)?;
                            let n = parse_num(t)?;
                            let DToken { token, line } = parse_next(itr.next(), line)?;
                            match token {
                                Token::ParenClose => {
                                    // indirect or y indexed
                                    if let Ok(DToken { token, line }) =
                                        parse_next(itr.next(), *line)
                                    {
                                        match token {
                                            Token::Comma => {
                                                parse_expect(
                                                    parse_next(itr.next(), *line)?,
                                                    Token::Y,
                                                )?;
                                                instruction_push!(
                                                    o,
                                                    AddressingMode::IND_Y,
                                                    Operand::U8(parse_u8(n, *line)?)
                                                );
                                            }
                                            Token::Newline => {
                                                // indirect
                                                instruction_push!(
                                                    o,
                                                    AddressingMode::IND,
                                                    Operand::U16(n)
                                                );
                                            }
                                            _ => {
                                                return Err(err!(
                                                    "unexpected symbol when trying to parse indirect/Y-indexed addressing",
                                                    *line
                                                ));
                                            }
                                        }
                                    } else {
                                        //  indirect
                                        instruction_push!(o, AddressingMode::IND, Operand::U16(n));
                                    }
                                }
                                Token::Comma => {
                                    // indirect x addressed
                                    instruction_push!(
                                        o,
                                        AddressingMode::X_IND,
                                        Operand::U8(parse_u8(n, *line)?)
                                    );
                                    parse_expect(parse_next(itr.next(), *line)?, Token::X)?;
                                    parse_expect(
                                        parse_next(itr.next(), *line)?,
                                        Token::ParenClose,
                                    )?;
                                }
                                t => {
                                    return Err(err!(
                                        format!(
                                            "unexpected token '{:?}' when trying to parse any of the indirect addressing modes",
                                            t
                                        ),
                                        *line
                                    ));
                                }
                            }
                        }
                        Token::A => {
                            // Accumulator addr-mode
                            let o = parse_opcode(i, line)?;
                            instruction_push!(o, AddressingMode::A, Operand::No);
                        }
                        Token::Newline => {
                            // Implied
                            already_found_newline = true;
                            let o = parse_opcode(i, line)?;
                            instruction_push!(o, AddressingMode::IMPL, Operand::No);
                        }
                        _ => return Err(err!("wrong token after ident", line)),
                    }
                } else {
                    let o = parse_opcode(i, line)?;
                    instruction_push!(o, AddressingMode::IMPL, Operand::No);
                }
            }
            Token::Newline => {
                already_found_newline = true;
            }
            t => {
                return Err(err!(
                    format!(
                        "expected either an instruction, a directive or a label, got token '{:?}'",
                        t
                    ),
                    line
                ));
            }
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
    if bank >= inesprg + ineschr {
        return Err(err!(format!("bank {bank} does not exist"), line_number));
    }
    use std::cmp::min;
    let offset = min(bank, inesprg) * 1024 * 16
        + if bank > inesprg {
            (bank - inesprg) * 1024 * 8
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
            }}
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
