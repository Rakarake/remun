use crate::*;
use shared::*;

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
