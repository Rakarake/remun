// Super amazing NES assembler.
// Zero-Page instructions are prefixed with an '@' before
// the rest of the operand.

use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::alpha1,
    character::complete::alphanumeric1,
    character::complete::char,
    character::complete::multispace0,
    character::complete::multispace1,
    character::complete::one_of,
    character::complete::space0,
    character::complete::space1,
    character::complete::u16,
    character::complete::u8,
    combinator::eof,
    combinator::map_res,
    combinator::opt,
    combinator::recognize,
    combinator::success,
    combinator::value,
    error::ParseError,
    multi::many0,
    multi::many0_count,
    multi::many1,
    sequence::pair,
    sequence::tuple,
    sequence::{preceded, terminated},
    IResult, Parser,
};

use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::{final_parser, Location, RecreateContext};
use nom_supreme::tag::complete::tag;
use nom_supreme::tag::complete::tag_no_case;

use nom_supreme::parser_ext::ParserExt;

// The amazing macro
use macros::funny_number;
use macros::from_table;

//use crate::nes_primitives::Opcodes;
//use crate::nes_primitives::Operands;

//funny_number!();
//const X: i32 = from_table!();

#[derive(Debug)]
enum Statement {
    Operation(Opcode, Operand),
    // Directives
    Bing,
    //DataBytes(Vec<u8>),
}

#[derive(Debug)]
enum DecoratedStatement {
    Label(String, Statement),
    NoLabel(Statement),
}

#[derive(Debug)]
enum Program {
    Program(Vec<DecoratedStatement>),
}

use nom_supreme::multi::collect_separated_terminated;

// Parse an entire string of assembly
fn parse_program(i: &str) -> IResult<&str, Program, ErrorTree<&str>> {
    // TODO: add support for newlines
    let (i, _) = parse_multispace_comment(i)?;
    let (i, statements) = collect_separated_terminated(
        parse_decorated_statement,
        parse_multispace_comment,
        pair(parse_multispace_comment, eof),
    )
    .parse(i)?;
    Ok((i, Program::Program(statements)))
}

// Use this to get a normal rust result
fn parse_program_final(i: &str) -> Result<Program, ErrorTree<&str>> {
    final_parser(parse_program)(i)
}

// A decorated statement might have a label
fn parse_decorated_statement(i: &str) -> IResult<&str, DecoratedStatement, ErrorTree<&str>> {
    let (i, maybe_label) = opt(terminated(parse_identifier, char(':')))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, statement) = parse_statement(i)?;
    let decorated_statement = match maybe_label {
        Some(label) => DecoratedStatement::Label(label.to_string(), statement),
        None => DecoratedStatement::NoLabel(statement),
    };
    Ok((i, decorated_statement))
}

fn parse_statement(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    let (i, _) = space0(i)?;
    // TODO: add parse_operation
    let (i, statement) = alt((
        parse_directive.context("directive"),
        parse_operation.context("operation"),
    ))(i)?;
    let (i, _) = alt((parse_comment, value((), space0)))(i)?;
    Ok((i, statement))
}

fn parse_directive(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    let (i, _) = char('.')(i)?;
    // TODO: add directives here with the alt combinator
    parse_bing_chilling(i)
}

// Directives

// The greatest test directive, gives off chills in the assembly
fn parse_bing_chilling(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    let (i, (_, _, _)) = tuple((tag("bing"), space0, tag("\"chilling\"")))(i)?;
    Ok((i, Statement::Bing))
}

// Operation = Opcode + Operand
fn parse_operation(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    // Opcode | Operand
    let (i, opcode) = parse_opcode(i)?;
    let (i, operand) = alt((
        preceded(space1, parse_operand(opcode.clone())),
        success(Operand::Implied),
    ))(i)?;

    Ok((i, Statement::Operation(opcode, operand)))
}

// Operands
fn parse_operand(opcode: Opcode) -> Box<dyn Fn(&str) -> IResult<&str, Operand, ErrorTree<&str>>> {
    // Match on opcode
    Box::new(move |i: &str| match opcode {
        Opcode::LDA => parse_all_addressing_modes(i),
        Opcode::STA => parse_all_addressing_modes(i),
        Opcode::NOP => parse_implied(i),
        Opcode::BNE => parse_relative(i),
        Opcode::ADC => parse_all_addressing_modes(i),
        Opcode::AND => parse_all_addressing_modes(i),
        Opcode::ASL => parse_shift_addressing_modes(i),
        Opcode::LSR => parse_shift_addressing_modes(i),
        Opcode::ROL => parse_shift_addressing_modes(i),
        Opcode::ROR => parse_shift_addressing_modes(i),
        Opcode::BIT => alt((parse_zero_page, parse_absolute))(i),
        Opcode::BRK => parse_implied(i),
        Opcode::BPL => parse_relative(i),
        Opcode::BMI => parse_relative(i),
        Opcode::BVC => parse_relative(i),
        Opcode::BVS => parse_relative(i),
        Opcode::BCC => parse_relative(i),
        Opcode::BCS => parse_relative(i),
        Opcode::BEQ => parse_relative(i),
        Opcode::CMP => parse_all_addressing_modes(i),
        Opcode::CPX => alt((parse_immediate, parse_zero_page, parse_absolute))(i),
        Opcode::CPY => alt((parse_immediate, parse_zero_page, parse_absolute))(i),
        Opcode::DEC => alt((parse_zero_page, parse_zero_page_x, parse_absolute, parse_absolute_x))(i),
        Opcode::INC => alt((parse_zero_page, parse_zero_page_x, parse_absolute, parse_absolute_x))(i),
        Opcode::EOR => parse_all_addressing_modes(i),
    })
}

// Addressing modes
//Implied,        //
//Immediate(u8),  //#$44
//Absolute(u16),  //$4400
//AbsoluteX(u16), //$4400,X
//AbsoluteY(u16), //$4400,Y
//ZeroPage(u8),   //$44
//ZeroPageX(u8),  //$44,X
//IndirectY(u8), //($44),Y
//IndirectX(u8), //($44,X)

// Parse all addressing modes except for implied 😀
fn parse_all_addressing_modes(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    alt((
        parse_immediate,
        parse_absolute_x,
        parse_absolute_y,
        parse_absolute,
        parse_zero_page_x,
        parse_zero_page,
        parse_indirect_x,
        parse_indirect_y,
    ))(i)
}

fn parse_shift_addressing_modes(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    alt((
        parse_accumulator,
        parse_absolute_x,
        parse_absolute,
        parse_zero_page_x,
        parse_zero_page,
    ))(i)
}


fn parse_implied(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    Ok((i, Operand::Implied))
}

fn parse_relative(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    alt((
        map_res(parse_u8, |num: u8| {
            Ok::<Operand, &str>(Operand::Relative(RelativeVal::Number(num)))
        }),
        map_res(parse_identifier, |identifier: &str| {
            Ok::<Operand, &str>(Operand::Relative(RelativeVal::Label(
                identifier.to_string(),
            )))
        }),
    ))(i)
}

fn parse_accumulator(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    value(Operand::Accumulator, char('A'))(i)
}

fn parse_immediate(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = char('#')(i)?;
    let (i, val) = parse_u8.context("immediate expecting u8").parse(i)?;
    Ok((i, Operand::Immediate(val)))
}

fn parse_zero_page(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = char('@')(i)?;
    let (i, val) = parse_u8.context("zero page expecting u8").parse(i)?;
    Ok((i, Operand::ZeroPage(val)))
}

fn parse_zero_page_x(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = char('@')(i)?;
    let (i, val) = parse_u8.context("zero page x expecting u8").parse(i)?;
    let (i, _) = parse_trailing_x_y("X")(i)?;
    Ok((i, Operand::ZeroPageX(val)))
}

fn parse_absolute(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, val) = parse_u16.context("absolute expecting u16").parse(i)?;
    Ok((i, Operand::Absolute(val)))
}

fn parse_absolute_x(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, val) = parse_u16.context("absolute x expecting u16").parse(i)?;
    let (i, _) = parse_trailing_x_y("X")(i)?;
    Ok((i, Operand::AbsoluteX(val)))
}

fn parse_absolute_y(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, val) = parse_u16.context("absolute y expecting u16").parse(i)?;
    let (i, _) = parse_trailing_x_y("Y")(i)?;
    Ok((i, Operand::AbsoluteY(val)))
}

fn parse_indirect_x(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = char('(')(i)?;
    let (i, val) = parse_u8.context("indirect x expecting u8").parse(i)?;
    let (i, _) = parse_trailing_x_y("X")(i)?;
    let (i, _) = space0(i)?;
    let (i, _) = char(')')(i)?;
    Ok((i, Operand::IndirectX(val)))
}

fn parse_indirect_y(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = char('(')(i)?;
    let (i, val) = parse_u8.context("indirect y expecting u8").parse(i)?;
    let (i, _) = char(')')(i)?;
    let (i, _) = parse_trailing_x_y("Y")(i)?;
    let (i, _) = space0(i)?;
    Ok((i, Operand::IndirectY(val)))
}

//IndirectX(u16), //($44,X)
//IndirectY(u16), //($44),Y

fn parse_trailing_x_y(
    x_or_y: &'static str,
) -> Box<dyn Fn(&str) -> IResult<&str, (), ErrorTree<&str>>> {
    Box::new(move |i: &str| {
        let (i, _) = space0(i)?;
        let (i, _) = char(',')(i)?;
        let (i, _) = space0(i)?;
        let (i, _) = tag_no_case(x_or_y)(i)?;
        Ok((i, ()))
    })
}

// Opcodes
fn parse_opcode(i: &str) -> IResult<&str, Opcode, ErrorTree<&str>> {
    alt((
        value(Opcode::LDA, tag("LDA")),
        value(Opcode::STA, tag("STA")),
        value(Opcode::NOP, tag("NOP")),
        value(Opcode::BNE, tag("BNE")),
        value(Opcode::ADC, tag("ADC")),
        value(Opcode::AND, tag("AND")),
        value(Opcode::ASL, tag("ASL")),
        value(Opcode::LSR, tag("LSR")),
        value(Opcode::ROL, tag("ROL")),
        value(Opcode::ROR, tag("ROR")),
        value(Opcode::BIT, tag("BIT")),
        value(Opcode::BRK, tag("BRK")),
        value(Opcode::BPL, tag("BPL")),
        value(Opcode::BMI, tag("BMI")),
        value(Opcode::BVC, tag("BVC")),
        value(Opcode::BVS, tag("BVS")),
        value(Opcode::BCC, tag("BCC")),
        value(Opcode::BCS, tag("BCS")),
        value(Opcode::BEQ, tag("BEQ")),
        value(Opcode::CMP, tag("CMP")),
        alt((
            value(Opcode::CPX, tag("CPX")),
            value(Opcode::CPY, tag("CPY")),
            value(Opcode::DEC, tag("DEC")),
            value(Opcode::INC, tag("INC")),
            value(Opcode::EOR, tag("EOR")),
        ))
    ))(i)
}

// Utility

// Any number of spaces, newlines and comment
fn parse_multispace_comment(i: &str) -> IResult<&str, (), ErrorTree<&str>> {
    let (i, _) = many0(alt((value((), multispace1), parse_comment)))(i)?;
    Ok((i, ()))
}

// Taken from the nom documentation
// https://docs.rs/nom/latest/nom/recipes/index.html#comments
fn parse_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (), // Output is thrown away.
        pair(char(';'), is_not("\n\r")),
    )(i)
}

// Modified from the nom documentation:
// https://docs.rs/nom/latest/nom/recipes/index.html#hexadecimal
fn parse_hex(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    preceded(
        char('$'),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )(i)
}

fn parse_bin(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    preceded(
        char('%'),
        recognize(many1(terminated(one_of("01"), many0(char('_'))))),
    )(i)
}

//TODO: add digit checking, bases 16=1-2, 10=1-3, 2=1-8
fn parse_u8(i: &str) -> IResult<&str, u8, ErrorTree<&str>> {
    alt((
        map_res(
            // Base 16
            parse_hex,
            |out: &str| u8::from_str_radix(&str::replace(&out, "_", ""), 16),
        ),
        map_res(
            // Base 2
            parse_bin,
            |out: &str| u8::from_str_radix(&str::replace(&out, "_", ""), 2),
        ),
        u8, // Base 10
    ))(i)
}

//TODO: add digit checking, bases 16=3-4, 10=4-?, 2=9-16
fn parse_u16(i: &str) -> IResult<&str, u16, ErrorTree<&str>> {
    alt((
        map_res(
            // Base 16
            parse_hex,
            |out: &str| u16::from_str_radix(&str::replace(&out, "_", ""), 16),
        ),
        map_res(
            // Base 2
            parse_bin,
            |out: &str| u16::from_str_radix(&str::replace(&out, "_", ""), 2),
        ),
        u16, // Base 10
    ))(i)
}

// Taken from the nom documentation:
// https://docs.rs/nom/latest/nom/recipes/index.html#rust-style-identifiers
pub fn parse_identifier(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(i)
}

fn main() {
    let result = parse_program_final(TEST);
    if let Err(e) = result {
        println!("Failiure!");
        //TODO: make a string and use it
        get_error_string(e);
    } else {
        println!("Success!");
        let _ = dbg!(result);
    }
}

//TODO: make a string and use it
use std::error::Error;
fn get_error_string(
    error: nom_supreme::error::GenericErrorTree<&str, &str, &str, Box<dyn Error + Send + Sync>>,
) {
    match error {
        nom_supreme::error::GenericErrorTree::Stack { base, contexts } => {
            println!("Stack");
            dbg!(base);
            for context in contexts {
                dbg!(Location::recreate_context(TEST, context.0));
                dbg!(context.1);
            }
        }
        nom_supreme::error::GenericErrorTree::Alt(inner_errors) => {
            println!("Alt");
            for inner_error in inner_errors {
                get_error_string(inner_error);
            }
        }
        nom_supreme::error::GenericErrorTree::Base { location, kind } => {
            println!("Base");
            dbg!(location, kind);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_addressing_modes() {
        let result = super::parse_program_final(TEST_ADDRESSING_MODES);
        match result {
            Err(error) => {
                super::get_error_string(error);
                panic!()
            }
            Ok(_) => {
                //for statement in statements {
                //    //TODO: check that every addressing mode is used
                //}
            }
        }
    }

    #[test]
    fn test_features() {
        let result = super::parse_program_final(TEST_FEATURES);
        match result {
            Err(error) => {
                super::get_error_string(error);
                panic!()
            }
            Ok(_) => {}
        }
    }

    const TEST_ADDRESSING_MODES: &str = "
IMPLIED:    NOP
RELATIVE:    BNE $50
RELATIVE:    BNE IMPLIED
IMMEDIATE:   LDA #$44
ZERO_PAGE:   LDA @$44
ZERO_PAGE_X: LDA @$44,X
ABSOLUTE:    LDA $4400
ABSOLUTE_X:  LDA $4400,X
ABSOLUTE_Y:  LDA $4400,Y
RELATIVE_X:  LDA ($44, X)
RELATIVE_Y:  LDA ($44), Y

; TODO: add relative addressing modes";

    const TEST_FEATURES: &str = "
BingChilling: .bing \"chilling\" ; The directive
;  Comment                          A comment
 ; Comment                          Another comment
LDA $30                           ; An instruction
 NOP                              ; Another instruction
LABEL1: LDA $30                   ; A label";
}

const TEST: &str = "
wowzers:   NOP

RELATIVE:    BNE $50
RELATIVE:    BNE IMPLIED

;ERROR:      LDA ##$3000
WOWZERS2:   LDA $2000,X        ; Big comment
            LDA ($20, X)
            LDA ($2F), Y
ZEROPAGEX:  LDA @$FF, X
            LDA $0000,Y
lad:        LDA #$30";
