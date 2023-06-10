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
  sequence::tuple,
  combinator::opt,
  multi::many0,
  combinator::all_consuming,
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

#[derive(Debug)]
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

#[derive(Debug)]
enum Opcode {
    LDA,
    STA,
}

#[derive(Debug)]
enum Statement {
    Operation(Opcode, Option<Operand>),
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
fn parse_program(input: &str) -> IResult<&str, Program> {
    // TODO: add support for newlines
    let (input, _) = parse_multispace_comment(input)?;
    let (input, statements) = all_consuming(many0(parse_decorated_statement))(input)?;
    Ok((input, Program::Program(statements)))
}

fn parse_label(input: &str) -> IResult<&str, &str> {
    let (input, label) = parse_identifier(input)?;
    let (input, _) = tag(":")(input)?;
    Ok((input, label))
}

// Taken from the nom documentation: 
// https://docs.rs/nom/latest/nom/recipes/index.html#rust-style-identifiers
pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
  recognize(
    pair(
      alt((alpha1, tag("_"))),
      many0_count(alt((alphanumeric1, tag("_"))))
    )
  )(input)
}

// A decorated statement might have a label
fn parse_decorated_statement(input: &str) -> IResult<&str, DecoratedStatement> {
    let (input, maybe_label) = opt(parse_label)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, statement) = parse_statement(input)?;
    let (input, _) = parse_multispace_comment(input)?;
    let decorated_statement = match maybe_label {
        Some(label) => DecoratedStatement::Label(label.to_string(), statement),
        None => DecoratedStatement::NoLabel(statement)
    };
    Ok((input, decorated_statement))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = space0(input)?;
    // TODO: add parse_operation
    let (input, statement) = alt((parse_directive,parse_directive))(input)?;
    let (input, _) = alt((parse_comment, value((), space0)))(input)?;
    Ok((input, statement))
}

// Any number of spaces, newlines and comment
fn parse_multispace_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((value((), multispace1), parse_comment)))(input)?;
    Ok((input, ()))
}

// Taken from the nom documentation
// https://docs.rs/nom/latest/nom/recipes/index.html#comments
fn parse_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
  value(
    (), // Output is thrown away.
    pair(char('%'), is_not("\n\r"))
  )(i)
}

fn parse_directive(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag(".")(input)?;
    // TODO: add directives here with the alt combinator
    parse_bing_chilling(input)
}

// The greatest test directive, gives off chills in the assembly
fn parse_bing_chilling (input: &str) -> IResult<&str, Statement> {
    let (input, (_, _, _)) = tuple((tag("bing"), space0, tag("\"chilling\"")))(input)?;
    Ok((input, Statement::Bing))
}

fn parse_operation(input: &str) -> IResult<&str, Statement> {
    !unimplemented!()
}

const TEST_1: &str = "#2F14DF";

const TEST_2: &str = "
_SUPBRU_H: .bing \"chilling\"
           .bing \"chilling\"
MONSer:    .bing \"chilling\"
";
    //LDA $40
    //TAX             ; Transfer A to X
    //STX $FFFF,X
fn main() {
  println!("{:?}", parse_program(TEST_2))
}

#[test]
fn parse_color() {
}

