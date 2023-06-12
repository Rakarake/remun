use nom::{
  bytes::complete::tag,
  IResult,
  error::ParseError,
  combinator::value,
  bytes::complete::is_not,
  character::complete::char,
  multi::many0_count,
  combinator::recognize,
  sequence::pair,
  character::complete::alpha1,
  branch::alt,
  character::complete::space0,
  character::complete::multispace0,
  character::complete::multispace1,
  character::complete::alphanumeric1,
  character::complete::u8,
  character::complete::u16,
  sequence::tuple,
  combinator::opt,
  multi::many0,
  combinator::all_consuming,
  multi::many1,
  sequence::{preceded, terminated},
  character::complete::one_of,
  combinator::map_res,
};

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
fn parse_program(i: &str) -> IResult<&str, Program> {
    // TODO: add support for newlines
    let (i, _) = parse_multispace_comment(i)?;
    let (i, statements) = all_consuming(many0(parse_decorated_statement))(i)?;
    Ok((i, Program::Program(statements)))
}

fn parse_label(i: &str) -> IResult<&str, &str> {
    let (i, label) = parse_identifier(i)?;
    let (i, _) = tag(":")(i)?;
    Ok((i, label))
}

// A decorated statement might have a label
fn parse_decorated_statement(i: &str) -> IResult<&str, DecoratedStatement> {
    let (i, maybe_label) = opt(parse_label)(i)?;
    let (i, _) = multispace0(i)?;
    let (i, statement) = parse_statement(i)?;
    let (i, _) = parse_multispace_comment(i)?;
    let decorated_statement = match maybe_label {
        Some(label) => DecoratedStatement::Label(label.to_string(), statement),
        None => DecoratedStatement::NoLabel(statement)
    };
    Ok((i, decorated_statement))
}

fn parse_statement(i: &str) -> IResult<&str, Statement> {
    let (i, _) = space0(i)?;
    // TODO: add parse_operation
    let (i, statement) = alt((parse_directive, parse_operation))(i)?;
    let (i, _) = alt((parse_comment, value((), space0)))(i)?;
    Ok((i, statement))
}

fn parse_directive(i: &str) -> IResult<&str, Statement> {
    let (i, _) = tag(".")(i)?;
    // TODO: add directives here with the alt combinator
    parse_bing_chilling(i)
}

// The greatest test directive, gives off chills in the assembly
fn parse_bing_chilling (i: &str) -> IResult<&str, Statement> {
    let (i, (_, _, _)) = tuple((tag("bing"), space0, tag("\"chilling\"")))(i)?;
    Ok((i, Statement::Bing))
}

fn parse_operation(i: &str) -> IResult<&str, Statement> {
    // Opcode | Operand
    let (i, opcode) = parse_opcode(i)?;
    let (i, operand) = parse_operand(i)?;
    !unimplemented!()
}

//Immediate     LDA #$44      $A9  2   2
//Zero Page     LDA $44       $A5  2   3
//Zero Page,X   LDA $44,X     $B5  2   4
//Absolute      LDA $4400     $AD  3   4
//Absolute,X    LDA $4400,X   $BD  3   4+
//Absolute,Y    LDA $4400,Y   $B9  3   4+
//Indirect,X    LDA ($44,X)   $A1  2   6
//Indirect,Y    LDA ($44),Y   $B1  2   5+
//enum Operand {
//    // Different adressing modes
//    Implied,        //
//    Immediate(u8),  //#$44
//    ZeroPage(u8),   //$44
//    ZeroPageX(u8),  //$44,X
//    Absolute(u16),  //$4400
//    AbsoluteX(u16), //$4400,X
//    AbsoluteY(u16), //$4400,Y
//    IndirectX(u16), //($44,X)
//    IndirectY(u16), //($44),Y
//}
fn parse_operand(i: &str) -> IResult<&str, Operand> {
    alt((
        value(Operand::Immediate(8), tag("\n")),
    ))(i)?;
    !unimplemented!()
}

fn parse_immediate(i: &str) -> IResult<&str, Operand> {
    let (i, _) = tag("#")(i)?;
    let (i, val) = parse_u8(i)?;
    Ok((i, Operand::Immediate(val)))
}

fn parse_opcode(i: &str) -> IResult<&str, Opcode> {
    alt((
        value(Opcode::LDA, tag("LDA")),
        value(Opcode::STA, tag("STA"))
    ))(i)
}

const TEST_1: &str = "#2F14DF";

const TEST_2: &str = "
_SUPBRU_H: LDA $40
           .bing \"chilling\"
MONSer:    .bing \"chilling\"
";
    //TAX             ; Transfer A to X
    //STX $FFFF,X
fn main() {
  println!("{:?}", parse_program(TEST_2))
}

#[test]
fn parse_color() {
}


// Utility

// Any number of spaces, newlines and comment
fn parse_multispace_comment(i: &str) -> IResult<&str, ()> {
    let (i, _) = many0(alt((value((), multispace1), parse_comment)))(i)?;
    Ok((i, ()))
}

// Taken from the nom documentation
// https://docs.rs/nom/latest/nom/recipes/index.html#comments
fn parse_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
  value(
    (), // Output is thrown away.
    pair(char('%'), is_not("\n\r"))
  )(i)
}

// Modified from the nom documentation: 
// https://docs.rs/nom/latest/nom/recipes/index.html#hexadecimal
fn parse_hex(i: &str) -> IResult<&str, &str> {
  preceded(
    tag("$"),
    recognize(
      many1(
        terminated(one_of("0123456789abcdefABCDEF"), many0(char('_')))
      )
    )
  )(i)
}

fn parse_bin(i: &str) -> IResult<&str, &str> {
  preceded(
    tag("%"),
    recognize(
      many1(
        terminated(one_of("01"), many0(char('_')))
      )
    )
  )(i)
}

fn parse_u8(i: &str) -> IResult<&str, u8> {
    alt((
        map_res(  // Base 16
          parse_hex,
          |out: &str| u8::from_str_radix(&str::replace(&out, "_", ""), 16)
        ),
        map_res(  // Base 2
          parse_bin,
          |out: &str| u8::from_str_radix(&str::replace(&out, "_", ""), 2)
        ),
        u8,  // Base 10
    ))(i)
}

fn parse_u16(i: &str) -> IResult<&str, u16> {
    alt((
        map_res(  // Base 16
          parse_hex,
          |out: &str| u16::from_str_radix(&str::replace(&out, "_", ""), 16)
        ),
        map_res(  // Base 2
          parse_bin,
          |out: &str| u16::from_str_radix(&str::replace(&out, "_", ""), 2)
        ),
        u16,  // Base 10
    ))(i)
}

// Taken from the nom documentation: 
// https://docs.rs/nom/latest/nom/recipes/index.html#rust-style-identifiers
pub fn parse_identifier(i: &str) -> IResult<&str, &str> {
  recognize(
    pair(
      alt((alpha1, tag("_"))),
      many0_count(alt((alphanumeric1, tag("_"))))
    )
  )(i)
}

