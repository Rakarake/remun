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
    combinator::all_consuming,
    combinator::map_res,
    combinator::opt,
    combinator::recognize,
    combinator::value,
    error::ParseError,
    multi::many0,
    multi::many0_count,
    multi::many1,
    sequence::pair,
    sequence::tuple,
    sequence::{preceded, terminated},
    IResult,
};

use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;

use nom_supreme::parser_ext::ParserExt;
use nom::error::context;

// Addressing modes for LDA

//Immediate     LDA #$44      $A9  2   2
//Zero Page     LDA $44       $A5  2   3
//Zero Page,X   LDA $44,X     $B5  2   4
//Absolute      LDA $4400     $AD  3   4
//Absolute,X    LDA $4400,X   $BD  3   4+
//Absolute,Y    LDA $4400,Y   $B9  3   4+
//Indirect,X    LDA ($44,X)   $A1  2   6
//Indirect,Y    LDA ($44),Y   $B1  2   5+

#[derive(Debug, Clone)]
enum Operand {
    // Different adressing modes
    Implied,        //
    Immediate(u8),  //#$44
    ZeroPage(u8),   //$44
    ZeroPageX(u8),  //$44,X
    Absolute(u16),  //$4400
    AbsoluteX(u16), //$4400,X
    AbsoluteY(u16), //$4400,Y
    IndirectX(u16), //($44,X)
    IndirectY(u16), //($44),Y
}

#[derive(Debug, Clone)]
enum Opcode {
    LDA,
    STA,
    NOP,
}

#[derive(Debug)]
enum Statement {
    Operation(Opcode, Operand),
    // Directives
    Bing,
    DataBytes(Vec<u8>),
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

// Parse an entire string of assembly
fn parse_program(i: &str) -> IResult<&str, Program, ErrorTree<&str>> {
    // TODO: add support for newlines
    let (i, _) = parse_multispace_comment(i)?;
    let (i, statements) = all_consuming(many0(parse_decorated_statement))(i)?;
    Ok((i, Program::Program(statements)))
}

fn parse_label(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (i, label) = parse_identifier(i)?;
    let (i, _) = tag(":")(i)?;
    Ok((i, label))
}

// A decorated statement might have a label
fn parse_decorated_statement(i: &str) -> IResult<&str, DecoratedStatement, ErrorTree<&str>> {
    let (i, maybe_label) = opt(parse_label)(i)?;
    let (i, _) = multispace0(i)?;
    let (i, statement) = parse_statement(i)?;
    let (i, _) = parse_multispace_comment(i)?;
    let decorated_statement = match maybe_label {
        Some(label) => DecoratedStatement::Label(label.to_string(), statement),
        None => DecoratedStatement::NoLabel(statement),
    };
    Ok((i, decorated_statement))
}

fn parse_statement(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    let (i, _) = space0(i)?;
    // TODO: add parse_operation
    let (i, statement) = alt((parse_directive, parse_operation))(i)?;
    let (i, _) = alt((parse_comment, value((), space0)))(i)?;
    Ok((i, statement))
}


fn parse_directive(i: &str) -> IResult<&str, Statement, ErrorTree<&str>> {
    let (i, _) = tag(".")(i)?;
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
    let (i, operand) = alt((preceded(space1, parse_operand(opcode.clone())), 
            value(Operand::Implied, parse_multispace_comment)))(i)?;
    Ok((i, Statement::Operation(opcode, operand)))
}

fn parse_operand(opcode: Opcode) -> Box<dyn Fn(&str) -> IResult<&str, Operand, ErrorTree<&str>>> {
    // Match on opcode
    Box::new(move|i: &str| match opcode {
        Opcode::LDA => alt((parse_immediate, parse_immediate))(i),
        Opcode::STA => parse_immediate(i),
        Opcode::NOP => parse_implied(i),
    })
}


// Operands

// Use last in 'alt'
fn parse_implied(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    Ok((i, Operand::Implied))
}

fn parse_immediate(i: &str) -> IResult<&str, Operand, ErrorTree<&str>> {
    let (i, _) = tag("#")(i)?;
    let (i, val) = parse_u8(i)?;
    Ok((i, Operand::Immediate(val)))
}

fn parse_opcode(i: &str) -> IResult<&str, Opcode, ErrorTree<&str>> {
    alt((
        value(Opcode::LDA, tag("LDA")),
        value(Opcode::STA, tag("STA")),
        value(Opcode::NOP, tag("NOP")),
    ))(i)
}

const TEST_1: &str = "#2F14DF";

const TEST_2: &str = "
wowzers:   NOP
wowzers2:  NOP
lad:       LDA #$30
";
//TAX             ; Transfer A to X
//STX $FFFF,X
fn main() {
    println!("{:?}", parse_program(TEST_2))
}

#[test]
fn parse_color() {}

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
        pair(char('%'), is_not("\n\r")),
    )(i)
}

// Modified from the nom documentation:
// https://docs.rs/nom/latest/nom/recipes/index.html#hexadecimal
fn parse_hex(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    preceded(
        tag("$"),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )(i)
}

fn parse_bin(i: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    preceded(
        tag("%"),
        recognize(many1(terminated(one_of("01"), many0(char('_'))))),
    )(i)
}

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
