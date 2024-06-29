use std::rc::Rc;

use super::{
    ast::{BinaryOp, Binding, UnuaryOp, Value},
    Iden, LNode, LNodeRef,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1},
    combinator::{cut, eof, map, map_res, opt, recognize, value},
    error::{context, ContextError, ParseError, VerboseError},
    multi::{fold_many0, many0, many0_count, many1, many1_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, Parser,
};

type LNodeResult<'a> = IResult<&'a str, LNodeRef, VerboseError<&'a str>>;

fn identifier(input: &str) -> IResult<&str, Iden, VerboseError<&str>> {
    let (rest, rec) = context(
        "ident",
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
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
        digit_str
            .parse::<i64>()
            .map(|e| Rc::new(LNode::Litteral(Value::Int(e))))
    })(input)
}

fn variable(input: &str) -> LNodeResult {
    let (rest, ident) = identifier(input)?;
    Ok((rest, Rc::new(LNode::Variable(ident))))
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
    Ok((
        rest,
        Rc::new(LNode::If {
            cond,
            then_do,
            else_do,
        }),
    ))
}

fn prefix_operator(input: &str) -> IResult<&str, UnuaryOp, VerboseError<&str>> {
    alt((
        value(UnuaryOp::IntNeg, char('-')),
        value(UnuaryOp::BoolNot, char('!')),
        value(UnuaryOp::StrToInt, tag("str2int")),
        value(UnuaryOp::IntNeg, tag("int2str")),
    ))(input)
}

fn prefix_expr(input: &str) -> LNodeResult {
    tuple((prefix_operator, expr))
        .map(|(op, body)| Rc::new(LNode::UnuaryOp { op, body }))
        .parse(input)
}

// either a ref to a variable, or a function call
fn core_expr(input: &str) -> LNodeResult {
    context(
        "core expr",
        alt((
            let_expr,
            paren_group_expr,
            prefix_expr,
            if_expr,
            integer_litteral,
            variable,
        )),
    )(input)
}

fn infix_operator(input: &str) -> IResult<&str, BinaryOp, VerboseError<&str>> {
    alt((
        value(BinaryOp::IntAdd, tag("+")),
        value(BinaryOp::IntSub, tag("-")),
        value(BinaryOp::IntMul, tag("*")),
        value(BinaryOp::IntDiv, tag("/")),
        value(BinaryOp::IntMod, tag("%")),
        value(BinaryOp::IntLt, tag("<")),
        value(BinaryOp::IntGt, tag(">")),
        value(BinaryOp::BoolOr, tag("|")),
        value(BinaryOp::BoolAnd, tag("&")),
        value(BinaryOp::StrConcat, tag(".")),
        value(BinaryOp::StrTake, tag("take")),
        value(BinaryOp::StrDrop, tag("drop")),
        value(BinaryOp::Eq, tag("==")),
    ))(input)
}

fn infix_expr(input: &str) -> LNodeResult {
    // core_expr [OP core_expr...]
    let (input, expr) = core_expr(input)?;
    fold_many0(
        tuple((delimited(sep_many0, infix_operator, sep_many0), core_expr)),
        move || expr.clone(),
        |res, (op, item)| {
            Rc::new(LNode::BinaryOp {
                op,
                left: res,
                right: item,
            })
        },
    )(input)
}

// a core expr or a function call
fn callseq_expr(input: &str) -> LNodeResult {
    // core_expr [core_expr...]
    let (input, expr) = infix_expr(input)?;
    fold_many0(
        preceded(sep_many1, infix_expr),
        move || expr.clone(),
        |res, item| {
            Rc::new(LNode::Apply {
                func: res,
                param: item,
            })
        },
    )(input)
}

fn expr(input: &str) -> LNodeResult {
    delimited(sep_many0, callseq_expr, sep_many0)(input)
}

fn top_expr(input: &str) -> LNodeResult {
    terminated(expr, eof)(input)
}

pub fn parse(input: &str) -> Result<LNodeRef, VerboseError<&str>> {
    let (_, res) = top_expr(input).finish()?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_integer() {
        let sample = r" 1 ";
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_comment() {
        let sample = r#"
            // a
            1
        "#;
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_variable() {
        let sample = r" a ";
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_apply_fancy() {
        let sample = r" f (g 1) b";
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_let_in_integer() {
        let sample = r#"
            let a = 1;
            in a
        "#;
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_function_call() {
        let sample = r#"
            let f a = a; in f 1
        "#;
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_if() {
        let sample = r#"
            if true { 1 } else { 2 }
        "#;
        let node = parse(sample).unwrap();
        println!("{:?}", node);
    }

    #[test]
    fn test_sub() {
        let sample = r#"
            x - 1
        "#;
        let node = parse(sample).unwrap();
        println!("{:#?}", node);
    }

    #[test]
    fn test_apply() {
        let sample = r#"
            a b
        "#;
        let node = parse(sample).unwrap();
        println!("{:#?}", node);
    }

    #[test]
    fn test_call_params() {
        let sample = r#"
            fac (x - 1)
        "#;
        let node = parse(sample).unwrap();
        println!("{:#?}", node);
    }

    #[test]
    fn test_mess() {
        let sample = r#"
            let a = 1;
                b = a + 1;
                f x y = x * y + b;
                rec fac x = if x < 2 { x } else { x * fac (x - 1) };
            in (f 2 a) + fac 3
        "#;
        let node = parse(sample).unwrap();
        println!("{:#?}", node);
    }
}
