use super::{
    ast::{BinaryOp, Binding, UnuaryOp},
    Iden, LNode, LNodeRef,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1, one_of},
    combinator::{cut, eof, map, map_res, opt, recognize, value, verify},
    error::{context, ContextError, ParseError, VerboseError},
    multi::{fold_many0, many0, many0_count, many1, many1_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, Parser,
};

type LNodeResult<'a> = IResult<&'a str, LNodeRef, VerboseError<&'a str>>;

fn identifier(input: &str) -> IResult<&str, Iden, VerboseError<&str>> {
    let (rest, rec) = context(
        "ident",
        verify(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_")))),
            )),
            |iden: &str| iden != "take" && iden != "drop",
        ),
    )
    .parse(input)?;
    Ok((rest, Iden::new(rec.to_owned())))
}

fn binding(input: &str) -> IResult<&str, Binding, VerboseError<&str>> {
    // [rec] name [param...] = value
    context(
        "binding",
        map(
            tuple((
                // [rec]
                opt(preceded(sep_many1, tag("rec"))),
                // name
                preceded(sep_many1, identifier),
                // [params...]
                many0(preceded(sep_many1, identifier)),
                // = value
                preceded(sep_many1, delimited(char('='), cut(expr), char(';'))),
            )),
            |(rec, id, params, expr)| Binding::new(rec.is_some(), id, params, expr),
        ),
    )(input)
}

fn let_expr(input: &str) -> LNodeResult {
    let (rest, (bindings, body)) = context(
        "let",
        tuple((
            preceded(tag("let"), cut(many1(binding))),
            cut(preceded(preceded(sep_many1, tag("in")), expr)),
        )),
    )(input)?;
    Ok((rest, LNode::Let { bindings, body }.into()))
}

fn paren_group_expr(input: &str) -> LNodeResult {
    delimited(tag("("), cut(expr), tag(")")).parse(input)
}

pub fn single_line_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (), // Output is thrown away.
        pair(tag("//"), is_not("\n\r")),
    )
    .parse(i)
}

fn sep<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    alt((value((), multispace1), single_line_comment))(input)
}

fn sep_many0<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (), E> {
    context("sep many0", value((), many0_count(sep)))(input)
}

fn sep_many1<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (), E> {
    context("sep many1", value((), many1_count(sep)))(input)
}

// integers litterals are only positive. negative integers are created using unuary minus
fn integer_litteral(input: &str) -> LNodeResult {
    map_res(digit1, |digit_str: &str| {
        digit_str.parse::<i64>().map(LNode::int)
    })(input)
}

fn boolean_litteral(input: &str) -> LNodeResult {
    alt((value(true, tag("true")), value(false, tag("false"))))
        .map(LNode::bool)
        .parse(input)
}

fn string_litteral(input: &str) -> LNodeResult {
    delimited(char('"'), cut(many0(alt((
        preceded(char('\\'), cut(one_of("\"\\"))),
        one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%&'()*+,-./:;<=>?@[]^_`|~ \n"),
    ))).map(|r| {
        let s: String = r.into_iter().collect();
        LNode::str(s)
    })), char('"'))(input)
}

fn variable(input: &str) -> LNodeResult {
    let (rest, ident) = identifier(input)?;
    Ok((rest, LNode::var(ident)))
}

fn braced_expr(input: &str) -> LNodeResult {
    delimited(
        sep_many0,
        context("braces", delimited(char('{'), cut(expr), char('}'))),
        sep_many0,
    )(input)
}

fn if_expr(input: &str) -> LNodeResult {
    let (rest, (cond, then_do, else_do)) = tuple((
        preceded(tag("if"), expr),
        cut(braced_expr),
        preceded(tag("else"), cut(braced_expr)),
    ))(input)?;
    Ok((rest, LNode::cond(cond, then_do, else_do)))
}

// either a ref to a variable, or a function call
fn core_expr(input: &str) -> LNodeResult {
    context(
        "core expr",
        alt((
            let_expr,
            paren_group_expr,
            if_expr,
            integer_litteral,
            boolean_litteral,
            string_litteral,
            variable,
        )),
    )(input)
}

// a core expr or a function call
fn callseq_expr(input: &str) -> LNodeResult {
    // core_expr [core_expr...]
    let (input, expr) = core_expr(input)?;
    fold_many0(
        preceded(sep_many1, core_expr),
        move || expr.clone(),
        LNode::apply,
    )(input)
}

#[derive(Clone, Copy)]
enum OperandOrder {
    Preserved,
    Reversed,
}

fn infix_operator(input: &str) -> IResult<&str, (BinaryOp, OperandOrder), VerboseError<&str>> {
    alt((
        value((BinaryOp::IntAdd, OperandOrder::Preserved), tag("+")),
        value((BinaryOp::IntSub, OperandOrder::Preserved), tag("-")),
        value((BinaryOp::IntMul, OperandOrder::Preserved), tag("*")),
        value((BinaryOp::IntDiv, OperandOrder::Preserved), tag("/")),
        value((BinaryOp::IntMod, OperandOrder::Preserved), tag("%")),
        value((BinaryOp::IntLt, OperandOrder::Preserved), tag("<")),
        value((BinaryOp::IntGt, OperandOrder::Preserved), tag(">")),
        value((BinaryOp::BoolOr, OperandOrder::Preserved), tag("|")),
        value((BinaryOp::BoolAnd, OperandOrder::Preserved), tag("&")),
        value((BinaryOp::StrConcat, OperandOrder::Preserved), tag(".")),
        value((BinaryOp::StrTake, OperandOrder::Reversed), tag("take")),
        value((BinaryOp::StrDrop, OperandOrder::Reversed), tag("drop")),
        value((BinaryOp::Eq, OperandOrder::Preserved), tag("==")),
    ))(input)
}

fn infix_expr(input: &str) -> LNodeResult {
    // core_expr [OP core_expr...]
    let (input, expr) = callseq_expr(input)?;
    fold_many0(
        tuple((
            delimited(sep_many0, infix_operator, sep_many0),
            callseq_expr,
        )),
        move || expr.clone(),
        |left, ((op, order), right)| match order {
            OperandOrder::Preserved => LNode::binary_op(op, left, right),
            OperandOrder::Reversed => LNode::binary_op(op, right, left),
        },
    )(input)
}

fn prefix_operator(input: &str) -> IResult<&str, UnuaryOp, VerboseError<&str>> {
    alt((
        value(UnuaryOp::IntNeg, char('-')),
        value(UnuaryOp::BoolNot, char('!')),
        value(UnuaryOp::StrToInt, tag("str2int")),
        value(UnuaryOp::IntNeg, tag("int2str")),
    ))(input)
}

fn expr(input: &str) -> LNodeResult {
    delimited(
        sep_many0,
        tuple((opt(prefix_operator), infix_expr)),
        sep_many0,
    )
    .map(|(op, expr)| {
        if let Some(op) = op {
            LNode::unuary_op(op, expr)
        } else {
            expr
        }
    })
    .parse(input)
}

fn top_expr(input: &str) -> LNodeResult {
    terminated(expr, eof)(input)
}

pub fn parse(input: &str) -> Result<LNodeRef, VerboseError<&str>> {
    let (_, res) = top_expr(input).finish()?;
    Ok(res)
}
