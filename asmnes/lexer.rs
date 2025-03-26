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
            if state != LexState::ReadingComment {
                delimiter(&mut state, line, &mut output, &mut acc)?;
                output.push(DToken {
                    token: $token,
                    line,
                });
            }
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
            ';' => if state != LexState::ReadingComment { state = LexState::ReadingComment },
            '$' => if state != LexState::ReadingComment {  state = LexState::ReadingHex },
            '%' => if state != LexState::ReadingComment {  state = LexState::ReadingBin },
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
