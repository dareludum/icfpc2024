mod ast;
mod compiler;
mod parser;

pub use ast::{Iden, LNode, LNodeRef};
pub use compiler::compile;
pub use parser::parse;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use num::BigInt;
    use num::FromPrimitive;

    use super::compile;
    use super::parse;
    use crate::icfp::evaluate;
    use crate::lasm::ast::BinaryOp;
    use crate::lasm::LNode;

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
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &10.into());
    }

    #[test]
    fn test_take() {
        let sample = r#"
            "ab" take 1
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        println!("{:#?}", evaluate(node));
        // assert_eq!(evaluate(node).as_str(), "a");
    }

    #[test]
    fn test_concat() {
        let sample = r#"
            let a = "a"; in
            a . "b"
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        println!("{:#?}", evaluate(node));
        // assert_eq!(evaluate(node).as_str(), "a");
    }


    #[test]
    fn test_equals() {
        let sample = r#"
            if 1 == 2 {
                3
            } else {
                4
            }
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &BigInt::from_u8(4).unwrap());
    }

    #[test]
    fn test_let_in() {
        let sample = r#"
            let a = 1;
                b = a + 1;
                f x y = x * y + b;
            in (f 2 a)
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &4.into());
    }

    #[test]
    fn test_string_litteral() {
        let sample = r#"
            "ab\"\\"
        "#;
        println!("{}", sample);
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_str(), "ab\"\\");
    }

    #[test]
    fn test_simple_string_litteral() {
        let sample = r#"
            "ab"
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_str(), "ab");
    }

    #[test]
    fn test_apply_unuary_precedence() {
        let sample = r#"
            let mul_two x = x * 2;
            in mul_two 1 - 1
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &1.into());
    }

    #[test]
    fn test_apply_mul_precedence() {
        let sample = r#"
            let mul_two x = x * 2;
            in 2 * mul_two 1
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &4.into());
    }

    #[test]
    fn test_infix_sequence() {
        let sample = r#"
            2 + 1 * 2 - 1
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &5.into());
    }

    #[test]
    fn test_rec() {
        let sample = r#"
            let rec fac x = if x < 2 { x } else { x * (fac (x - 1)) };
            in fac 3
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &6.into());
    }

    #[test]
    fn test_rec_prio() {
        let sample = r#"
            let rec fac x = if x < 2 { x } else { x * fac (x - 1) };
            in fac 3
        "#;
        let node = parse(sample).unwrap();
        let node = compile(node);
        assert_eq!(evaluate(node).as_int(), &6.into());
    }

    #[test]
    fn test_integer() {
        let sample = r" 1 ";
        let node = parse(sample).unwrap();
        assert_eq!(evaluate(compile(node)).as_int(), &1.into());
    }

    #[test]
    fn test_comment() {
        let sample = r#"
            // a
            1
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(evaluate(compile(node)).as_int(), &1.into());
    }

    #[test]
    fn test_variable() {
        let sample = r" a ";
        let node = parse(sample).unwrap();
        assert_eq!(node, LNode::var("a".to_owned()));
    }

    #[test]
    fn test_apply_fancy() {
        let sample = r" f (g 1) b";
        parse(sample).unwrap();
    }

    #[test]
    fn test_let_in_integer() {
        let sample = r#"
            let a = 1;
            in a
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(evaluate(compile(node)).as_int(), &1.into());
    }

    #[test]
    fn test_function_call() {
        let sample = r#"
            let f a = a; in f 1
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(evaluate(compile(node)).as_int(), &1.into());
    }

    #[test]
    fn test_if() {
        let sample = r#"
            if true { 1 } else { 2 }
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(evaluate(compile(node)).as_int(), &1.into());
    }

    #[test]
    fn test_sub() {
        let sample = r#"
            x - 1
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(
            Rc::new(LNode::BinaryOp {
                op: BinaryOp::IntSub,
                left: LNode::var("x".to_owned()),
                right: LNode::int(1.into()),
            }),
            node
        );
    }

    #[test]
    fn test_apply() {
        let sample = r#"
            a b
        "#;
        let node = parse(sample).unwrap();
        assert_eq!(
            LNode::apply(LNode::var("a".to_owned()), LNode::var("b".to_owned())),
            node
        );
    }

    #[test]
    fn test_call_params() {
        let sample = r#"
            fac (x - 1)
        "#;
        let node = parse(sample).unwrap();
        println!("{:#?}", node);
    }
}
