use nom::{
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::Tuple,
  IResult,
};

#[derive(Debug, PartialEq)]
pub struct Color {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = (hex_primary, hex_primary, hex_primary).parse(input)?;
  Ok((input, Color { red, green, blue }))
}

//Immediate     LDA #$44      $A9  2   2
//Zero Page     LDA $44       $A5  2   3
//Zero Page,X   LDA $44,X     $B5  2   4
//Absolute      LDA $4400     $AD  3   4
//Absolute,X    LDA $4400,X   $BD  3   4+
//Absolute,Y    LDA $4400,Y   $B9  3   4+
//Indirect,X    LDA ($44,X)   $A1  2   6
//Indirect,Y    LDA ($44),Y   $B1  2   5+

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

enum Opcode {
    LDA,
    STA,
}

enum Statement {
    Operation(Opcode, Option<Operand>),
    // Directives
    Bing,
    DataBytes(Vec<u8>),
}

enum DecoratedStatement {
    Label(String, Statement),
    NoLabel(Statement),
}

enum Program {
    Program(Vec<DecoratedStatement>),
}

use nom::branch::alt;
fn parse_program(input: &str) -> IResult<&str, Program> {
    unimplemented!()
}

use nom::character::complete::space0;
use nom::character::complete::multispace0;
use nom::character::complete::line_ending;
use nom::character::complete::alpha1;
use nom::sequence::tuple;
use nom::combinator::opt;
fn parse_label(input: &str) -> IResult<&str, &str> {
    let (input, label) = alpha1(input)?;
    let (input, _) = tag(":")(input)?;
    Ok((input, label))
}

fn parse_decorated_statement(input: &str) -> IResult<&str, DecoratedStatement> {
    let (input, maybe_label) = opt(parse_label)(input)?;
    let (input, statement) = parse_statement(input)?;
    let decorated_statement = match maybe_label {
        Some(label) => DecoratedStatement::Label(label.to_string(), statement),
        None => DecoratedStatement::NoLabel(statement)
    };
    Ok((input, decorated_statement))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = space0(input)?;
    let (input, statement) = alt((parse_operation,parse_directive))(input)?;
    let (input, _) = alt((parse_comment, value((), space0)))(input)?;
    Ok((input, statement))
}

fn parse_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
  value(
    (), // Output is thrown away.
    pair(char('%'), is_not("\n\r"))
  )(i)
}

fn parse_directive(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag(".")(input)?;
    !unimplemented!()
}

fn parse_operation(input: &str) -> IResult<&str, Statement> {
    !unimplemented!()
}

use nom::{
  error::ParseError,
  combinator::value,
  sequence::pair,
  bytes::complete::is_not,
  character::complete::char,
};

const TEST_1: &str = "#2F14DF";

const TEST_2: &str = "
    .bing \"chilling\"
    LDA $40
    TAX             ; Transfer A to X
    STX $FFFF,X
";

fn main() {
  println!("{:?}", hex_color(TEST_1))
}

#[test]
fn parse_color() {
  assert_eq!(
    hex_color(TEST_1),
    Ok((
      "",
      Color {
        red: 47,
        green: 20,
        blue: 223,
      }
    ))
  );
}

