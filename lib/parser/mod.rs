use nom::*;

pub mod ast;
use crate::lexer::token::*;
use crate::parser::ast::*;
use nom::branch::*;
use nom::bytes::complete::take;
use nom::combinator::{map, opt, verify};
use nom::error::ParseError;
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::*;
use nom::Err;
use std::result::Result::*;

pub(crate) fn match_text<'a, Error: ParseError<Tokens<'a>>>(
    text: &'a str,
) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, Tokens<'a>, Error> {
    move |i: Tokens<'a>| {
        let ret: IResult<Tokens<'a>, Tokens<'a>, Error> =
            verify(take(1usize), |_t: &Tokens<'a>| "test" == text)(i);
        ret
    }
}

pub(crate) fn match_token<'a, Error: ParseError<Tokens<'a>>>(
    kind: Token,
) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, Tokens<'a>, Error> {
    move |i: Tokens<'a>| {
        let ret: IResult<Tokens<'a>, Tokens<'a>, Error> =
            verify(take(1usize), |_t: &Tokens<'a>| _t.tok[0] == kind)(i);
        ret
    }
}

macro_rules! rule {
    ($($tt:tt)*) => {
        nom_rule::rule!(match_text, match_token, $($tt)*)
    }
}

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::IntLiteral(name) => Ok((i1, Literal::IntLiteral(name))),
            Token::StringLiteral(s) => Ok((i1, Literal::StringLiteral(s))),
            Token::BoolLiteral(b) => Ok((i1, Literal::BoolLiteral(b))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}
fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Ident(name) => Ok((i1, Ident(name))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn infix_op(t: &Token) -> (Precedence, Option<Infix>) {
    match *t {
        Token::Equal => (Precedence::PEquals, Some(Infix::Equal)),
        Token::NotEqual => (Precedence::PEquals, Some(Infix::NotEqual)),
        Token::LessThanEqual => (Precedence::PLessGreater, Some(Infix::LessThanEqual)),
        Token::GreaterThanEqual => (Precedence::PLessGreater, Some(Infix::GreaterThanEqual)),
        Token::LessThan => (Precedence::PLessGreater, Some(Infix::LessThan)),
        Token::GreaterThan => (Precedence::PLessGreater, Some(Infix::GreaterThan)),
        Token::Plus => (Precedence::PSum, Some(Infix::Plus)),
        Token::Minus => (Precedence::PSum, Some(Infix::Minus)),
        Token::Multiply => (Precedence::PProduct, Some(Infix::Multiply)),
        Token::Divide => (Precedence::PProduct, Some(Infix::Divide)),
        Token::LParen => (Precedence::PCall, None),
        Token::LBracket => (Precedence::PIndex, None),
        _ => (Precedence::PLowest, None),
    }
}

fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
    map(rule!(#parse_stmt* ~ Token::EOF), |(v, _)| v)(input)
}

fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    parse_pratt_expr(input, Precedence::PLowest)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        rule!( (#parse_let_stmt : "`Let statement`"
        | #parse_return_stmt : "`return statement`"
        | #parse_expr_stmt: "`exprestion statement`")),
        |o| o,
    )(input)
}

fn parse_let_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        rule!(Token::Let ~ #parse_ident ~ Token::Assign ~ #parse_expr ~ Token::SemiColon?),
        |(_, ident, _, expr, _)| Stmt::LetStmt(ident, expr),
    )(input)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        rule!( Token::Return ~ #parse_expr ~ Token::SemiColon? ),
        |(_, e, _)| Stmt::ReturnStmt(e),
    )(input)
}

fn parse_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(rule!(#parse_expr ~ Token::SemiColon?), |(expr, _)| {
        Stmt::ExprStmt(expr)
    })(input)
}

fn parse_block_stmt(input: Tokens) -> IResult<Tokens, Program> {
    map(rule!( Token::LBrace ~ #parse_stmt* ~ Token::RBrace), |o| {
        o.1
    })(input)
}

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    rule!(
            #parse_lit_expr : "Literal Expr"
           | #parse_ident_expr : "Ident Expr"
           | #parse_prefix_expr : "Prefix Expr"
           | #parse_paren_expr : "Paren Expr"
           | #parse_array_expr : "Array Expr"
           | #parse_hash_expr : "Hash Expr"
           | #parse_if_expr : "If Expr"
           | #parse_fn_expr : "Fn Expr"
    )(input)
}

fn parse_paren_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(rule!( Token::LParen ~ #parse_expr ~ Token::RParen), |o| o.1)(input)
}

fn parse_lit_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::LitExpr)(input)
}
fn parse_ident_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_ident, Expr::IdentExpr)(input)
}
fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
    map(rule!( Token::Comma ~ #parse_expr), |(_, e)| e)(input)
}
fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    map(
        rule!(#parse_expr ~ #parse_comma_exprs*),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}
fn empty_boxed_vec(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    Ok((input, vec![]))
}

fn parse_array_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        rule!( Token::LBracket ~ (#parse_exprs | #empty_boxed_vec) ~ Token::RBracket),
        |o| Expr::ArrayExpr(o.1),
    )(input)
}

fn parse_hash_pair(input: Tokens) -> IResult<Tokens, (Literal, Expr)> {
    map(
        rule!(#parse_literal~ Token::Colon ~ #parse_expr),
        |(e, _, v)| (e, v),
    )(input)
}

fn parse_hash_comma_expr(input: Tokens) -> IResult<Tokens, (Literal, Expr)> {
    map(rule!( Token::Comma ~ #parse_hash_pair), |(_, e)| e)(input)
}

fn parse_hash_pairs(input: Tokens) -> IResult<Tokens, Vec<(Literal, Expr)>> {
    map(
        rule!( #parse_hash_pair ~ #parse_hash_comma_expr*),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}
fn empty_pairs(input: Tokens) -> IResult<Tokens, Vec<(Literal, Expr)>> {
    Ok((input, vec![]))
}
fn parse_hash_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        rule!( Token::LBrace ~ ( #parse_hash_pairs | #empty_pairs ) ~ Token::RBrace ),
        |(_, e, _)| Expr::HashExpr(e),
    )(input)
}

fn parse_prefix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    let (i1, t1) = rule!(Token::Plus | Token::Minus | Token::Not)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(error_position!(input, ErrorKind::Tag)))
    } else {
        let (i2, e) = parse_atom_expr(i1)?;
        match t1.tok[0].clone() {
            Token::Plus => Ok((i2, Expr::PrefixExpr(Prefix::PrefixPlus, Box::new(e)))),
            Token::Minus => Ok((i2, Expr::PrefixExpr(Prefix::PrefixMinus, Box::new(e)))),
            Token::Not => Ok((i2, Expr::PrefixExpr(Prefix::Not, Box::new(e)))),
            _ => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
        }
    }
}

fn parse_pratt_expr(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, left) = parse_atom_expr(input)?;
    go_parse_pratt_expr(i1, precedence, left)
}

fn go_parse_pratt_expr(input: Tokens, precedence: Precedence, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;

    if t1.tok.is_empty() {
        Ok((i1, left))
    } else {
        let preview = &t1.tok[0];
        let p = infix_op(preview);
        match p {
            (Precedence::PCall, _) if precedence < Precedence::PCall => {
                let (i2, left2) = parse_call_expr(input, left)?;
                go_parse_pratt_expr(i2, precedence, left2)
            }
            (Precedence::PIndex, _) if precedence < Precedence::PIndex => {
                let (i2, left2) = parse_index_expr(input, left)?;
                go_parse_pratt_expr(i2, precedence, left2)
            }
            (ref peek_precedence, _) if precedence < *peek_precedence => {
                let (i2, left2) = parse_infix_expr(input, left)?;
                go_parse_pratt_expr(i2, precedence, left2)
            }
            _ => Ok((input, left)),
        }
    }
}

fn parse_infix_expr(input: Tokens, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(error_position!(input, ErrorKind::Tag)))
    } else {
        let next = &t1.tok[0];
        let (precedence, maybe_op) = infix_op(next);
        match maybe_op {
            None => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
            Some(op) => {
                let (i2, right) = parse_pratt_expr(i1, precedence)?;
                Ok((i2, Expr::InfixExpr(op, Box::new(left), Box::new(right))))
            }
        }
    }
}

fn parse_call_expr(input: Tokens, fn_handle: Expr) -> IResult<Tokens, Expr> {
    map(
        rule!( Token::LParen ~ ( #parse_exprs | #empty_boxed_vec ) ~ Token::RParen),
        |(_, e, _)| Expr::CallExpr {
            function: Box::new(fn_handle.clone()).clone(),
            arguments: e,
        },
    )(input)
}

fn parse_index_expr(input: Tokens, arr: Expr) -> IResult<Tokens, Expr> {
    map(
        rule!(Token::LBracket ~ #parse_expr ~ Token::RBracket),
        |o| Expr::IndexExpr {
            array: Box::new(arr.clone()).clone(),
            index: Box::new(o.1),
        },
    )(input)
}

fn parse_if_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        rule!( Token::If ~ Token::LParen ~ #parse_expr ~ Token::RParen ~ #parse_block_stmt ~ #parse_else_expr ),
        |(_, _, expr, _, c, a)| Expr::IfExpr {
            cond: Box::new(expr),
            consequence: c,
            alternative: a,
        },
    )(input)
}
fn parse_else_expr(input: Tokens) -> IResult<Tokens, Option<Program>> {
    map(
        rule!( ( Token::Else ~ #parse_block_stmt )?),
        |(b)| match b {
            Some(s) => Some(s.1),
            None => None,
        },
    )(input)
}
fn empty_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> {
    Ok((input, vec![]))
}
fn parse_fn_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        rule!( Token::Function ~ Token::LParen ~ (#parse_params | #empty_params) ~ Token::RParen ~ #parse_block_stmt),
        |(_, _, p, _, b)| Expr::FnExpr { params: p, body: b },
    )(input)
}
fn parse_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> {
    map(
        rule!( #parse_ident ~  ( Token::Comma ~ #parse_ident)*),
        |(p, ps)| {
            [
                &vec![p][..],
                &ps.into_iter().map(|(_, t)| t).collect::<Vec<_>>()[..],
            ]
            .concat()
        },
    )(input)
}

pub struct Parser;

impl Parser {
    pub fn parse_tokens(tokens: Tokens) -> IResult<Tokens, Program> {
        parse_program(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::*;

    fn assert_input_with_program(input: &[u8], expected_results: Program) {
        let (_, r) = Lexer::lex_tokens(input).unwrap();
        let tokens = Tokens::new(&r);
        let (_, result) = Parser::parse_tokens(tokens).unwrap();
        assert_eq!(result, expected_results);
    }

    fn compare_inputs(input: &[u8], input2: &[u8]) {
        let (_, r) = Lexer::lex_tokens(input).unwrap();
        let tokens = Tokens::new(&r);
        let (_, result) = Parser::parse_tokens(tokens).unwrap();

        let (_, r) = Lexer::lex_tokens(input2).unwrap();
        let tokens = Tokens::new(&r);
        let (_, expected_results) = Parser::parse_tokens(tokens).unwrap();

        assert_eq!(result, expected_results);
    }

    #[test]
    fn empty() {
        assert_input_with_program(&b""[..], vec![]);
    }

    #[test]
    fn let_statements() {
        let input = "let x = 5;\
             let y = 10;\
             let foobar = 838383;\
             let boo = true;\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::LetStmt(Ident("x".to_owned()), Expr::LitExpr(Literal::IntLiteral(5))),
            Stmt::LetStmt(
                Ident("y".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(10)),
            ),
            Stmt::LetStmt(
                Ident("foobar".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(838383)),
            ),
            Stmt::LetStmt(
                Ident("boo".to_owned()),
                Expr::LitExpr(Literal::BoolLiteral(true)),
            ),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn return_statements() {
        let input = "return 5;\
             return 10;\
             return 838383;\
             return true;\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::ReturnStmt(Expr::LitExpr(Literal::IntLiteral(5))),
            Stmt::ReturnStmt(Expr::LitExpr(Literal::IntLiteral(10))),
            Stmt::ReturnStmt(Expr::LitExpr(Literal::IntLiteral(838383))),
            Stmt::ReturnStmt(Expr::LitExpr(Literal::BoolLiteral(true))),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn some_statements() {
        let input = "let x = 5;\
             return 10;\
             15;\
             let y = 20;\
             return false;\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::LetStmt(Ident("x".to_owned()), Expr::LitExpr(Literal::IntLiteral(5))),
            Stmt::ReturnStmt(Expr::LitExpr(Literal::IntLiteral(10))),
            Stmt::ExprStmt(Expr::LitExpr(Literal::IntLiteral(15))),
            Stmt::LetStmt(
                Ident("y".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(20)),
            ),
            Stmt::ReturnStmt(Expr::LitExpr(Literal::BoolLiteral(false))),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn identifier() {
        let input = "foobar;\
             foobar\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::ExprStmt(Expr::IdentExpr(Ident("foobar".to_owned()))),
            Stmt::ExprStmt(Expr::IdentExpr(Ident("foobar".to_owned()))),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn prefix_expr() {
        let input = "-foobar;\
             +10\
             !true\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::PrefixMinus,
                Box::new(Expr::IdentExpr(Ident("foobar".to_owned()))),
            )),
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::PrefixPlus,
                Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
            )),
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::Not,
                Box::new(Expr::LitExpr(Literal::BoolLiteral(true))),
            )),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn prefix_expr2() {
        let input = "-(foobar);\
             (+(10));\
             (((!true)));\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::PrefixMinus,
                Box::new(Expr::IdentExpr(Ident("foobar".to_owned()))),
            )),
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::PrefixPlus,
                Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
            )),
            Stmt::ExprStmt(Expr::PrefixExpr(
                Prefix::Not,
                Box::new(Expr::LitExpr(Literal::BoolLiteral(true))),
            )),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn infix_expr() {
        let input = "10 + 20".as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::InfixExpr(
            Infix::Plus,
            Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
            Box::new(Expr::LitExpr(Literal::IntLiteral(20))),
        ))];

        assert_input_with_program(input, program);

        let input = "10 * 20".as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::InfixExpr(
            Infix::Multiply,
            Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
            Box::new(Expr::LitExpr(Literal::IntLiteral(20))),
        ))];

        assert_input_with_program(input, program);

        let input = "10 + 5 / -20 - (x + x)".as_bytes();

        let input2 = "10 + (5 / (-20)) - (x + x)".as_bytes();

        compare_inputs(input, input2);

        let input = "10 + 5 / -20 - (x + x)".as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::InfixExpr(
            Infix::Minus,
            Box::new(Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
                Box::new(Expr::InfixExpr(
                    Infix::Divide,
                    Box::new(Expr::LitExpr(Literal::IntLiteral(5))),
                    Box::new(Expr::PrefixExpr(
                        Prefix::PrefixMinus,
                        Box::new(Expr::LitExpr(Literal::IntLiteral(20))),
                    )),
                )),
            )),
            Box::new(Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
                Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
            )),
        ))];

        assert_input_with_program(input, program);
    }

    #[test]
    fn op_precedence() {
        let input = "!-a".as_bytes();

        let input2 = "(!(-a))".as_bytes();

        compare_inputs(input, input2);

        let input = "a + b + c".as_bytes();

        let input2 = "((a + b) + c)".as_bytes();

        compare_inputs(input, input2);

        let input = "a + b - c".as_bytes();

        let input2 = "((a + b) - c)".as_bytes();

        compare_inputs(input, input2);

        let input = "a * b * c".as_bytes();

        let input2 = "((a * b) * c)".as_bytes();

        compare_inputs(input, input2);

        let input = "a * b / c".as_bytes();

        let input2 = "((a * b) / c)".as_bytes();

        compare_inputs(input, input2);

        let input = "a + b / c".as_bytes();

        let input2 = "(a + (b / c))".as_bytes();

        compare_inputs(input, input2);

        let input = "a + b * c + d / e - f".as_bytes();

        let input2 = "(((a + (b * c)) + (d / e)) - f)".as_bytes();

        compare_inputs(input, input2);

        let input = "3 + 4; -5 * 5".as_bytes();

        let input2 = "(3 + 4);((-5) * 5)".as_bytes();

        compare_inputs(input, input2);

        let input = "5 > 4 == 3 < 4".as_bytes();

        let input2 = "((5 > 4) == (3 < 4))".as_bytes();

        compare_inputs(input, input2);

        let input = "5 < 4 != 3 > 4".as_bytes();

        let input2 = "((5 < 4) != (3 > 4))".as_bytes();

        compare_inputs(input, input2);

        let input = "3 + 4 * 5 == 3 * 1 + 4 * 5".as_bytes();

        let input2 = "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".as_bytes();

        compare_inputs(input, input2);
    }

    #[test]
    fn if_expr() {
        let input = "if (x < y) { x }".as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::IfExpr {
            cond: Box::new(Expr::InfixExpr(
                Infix::LessThan,
                Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
                Box::new(Expr::IdentExpr(Ident("y".to_owned()))),
            )),
            consequence: vec![Stmt::ExprStmt(Expr::IdentExpr(Ident("x".to_owned())))],
            alternative: None,
        })];

        assert_input_with_program(input, program);

        let input = "if (x < y) { x } else { y }".as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::IfExpr {
            cond: Box::new(Expr::InfixExpr(
                Infix::LessThan,
                Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
                Box::new(Expr::IdentExpr(Ident("y".to_owned()))),
            )),
            consequence: vec![Stmt::ExprStmt(Expr::IdentExpr(Ident("x".to_owned())))],
            alternative: Some(vec![Stmt::ExprStmt(Expr::IdentExpr(Ident("y".to_owned())))]),
        })];

        assert_input_with_program(input, program);
    }

    #[test]
    fn function_expr() {
        let input = "fn() {\
                return foobar + barfoo;\
            }\
            "
        .as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::FnExpr {
            params: vec![],
            body: vec![Stmt::ReturnStmt(Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::IdentExpr(Ident("foobar".to_owned()))),
                Box::new(Expr::IdentExpr(Ident("barfoo".to_owned()))),
            ))],
        })];

        assert_input_with_program(input, program);

        let input = "fn(x, y) {\
                return x + y;\
            }\
            "
        .as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::FnExpr {
            params: vec![Ident("x".to_owned()), Ident("y".to_owned())],
            body: vec![Stmt::ReturnStmt(Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
                Box::new(Expr::IdentExpr(Ident("y".to_owned()))),
            ))],
        })];

        assert_input_with_program(input, program);

        let input = "fn() {
                return fn (x, y, z, zz) { return x >= y; };
             }
            "
        .as_bytes();

        let program: Program = vec![Stmt::ExprStmt(Expr::FnExpr {
            params: vec![],
            body: vec![Stmt::ReturnStmt(Expr::FnExpr {
                params: vec![
                    Ident("x".to_owned()),
                    Ident("y".to_owned()),
                    Ident("z".to_owned()),
                    Ident("zz".to_owned()),
                ],
                body: vec![Stmt::ReturnStmt(Expr::InfixExpr(
                    Infix::GreaterThanEqual,
                    Box::new(Expr::IdentExpr(Ident("x".to_owned()))),
                    Box::new(Expr::IdentExpr(Ident("y".to_owned()))),
                ))],
            })],
        })];

        assert_input_with_program(input, program);
    }

    #[test]
    fn function_call_expr() {
        let input = "add(2, 3);\
             add(a, b, 1, 2 * 3, other(4 + 5), add(6, 7 * 8));\
             fn(a, b) { return a + b; }(1, 2);\
            "
        .as_bytes();

        let program: Program = vec![
            Stmt::ExprStmt(Expr::CallExpr {
                function: Box::new(Expr::IdentExpr(Ident("add".to_owned()))),
                arguments: vec![
                    Expr::LitExpr(Literal::IntLiteral(2)),
                    Expr::LitExpr(Literal::IntLiteral(3)),
                ],
            }),
            Stmt::ExprStmt(Expr::CallExpr {
                function: Box::new(Expr::IdentExpr(Ident("add".to_owned()))),
                arguments: vec![
                    Expr::IdentExpr(Ident("a".to_owned())),
                    Expr::IdentExpr(Ident("b".to_owned())),
                    Expr::LitExpr(Literal::IntLiteral(1)),
                    Expr::InfixExpr(
                        Infix::Multiply,
                        Box::new(Expr::LitExpr(Literal::IntLiteral(2))),
                        Box::new(Expr::LitExpr(Literal::IntLiteral(3))),
                    ),
                    Expr::CallExpr {
                        function: Box::new(Expr::IdentExpr(Ident("other".to_owned()))),
                        arguments: vec![Expr::InfixExpr(
                            Infix::Plus,
                            Box::new(Expr::LitExpr(Literal::IntLiteral(4))),
                            Box::new(Expr::LitExpr(Literal::IntLiteral(5))),
                        )],
                    },
                    Expr::CallExpr {
                        function: Box::new(Expr::IdentExpr(Ident("add".to_owned()))),
                        arguments: vec![
                            Expr::LitExpr(Literal::IntLiteral(6)),
                            Expr::InfixExpr(
                                Infix::Multiply,
                                Box::new(Expr::LitExpr(Literal::IntLiteral(7))),
                                Box::new(Expr::LitExpr(Literal::IntLiteral(8))),
                            ),
                        ],
                    },
                ],
            }),
            Stmt::ExprStmt(Expr::CallExpr {
                function: Box::new(Expr::FnExpr {
                    params: vec![Ident("a".to_owned()), Ident("b".to_owned())],
                    body: vec![Stmt::ReturnStmt(Expr::InfixExpr(
                        Infix::Plus,
                        Box::new(Expr::IdentExpr(Ident("a".to_owned()))),
                        Box::new(Expr::IdentExpr(Ident("b".to_owned()))),
                    ))],
                }),
                arguments: vec![
                    Expr::LitExpr(Literal::IntLiteral(1)),
                    Expr::LitExpr(Literal::IntLiteral(2)),
                ],
            }),
        ];

        assert_input_with_program(input, program);
    }

    #[test]
    fn strings() {
        let input = &b"\"foobar\""[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::LitExpr(Literal::StringLiteral(
            "foobar".to_owned(),
        )))];

        assert_input_with_program(input, program);

        let input = &b"\"foo bar\""[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::LitExpr(Literal::StringLiteral(
            "foo bar".to_owned(),
        )))];

        assert_input_with_program(input, program);

        let input = &b"\"foo\nbar\""[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::LitExpr(Literal::StringLiteral(
            "foo\nbar".to_owned(),
        )))];

        assert_input_with_program(input, program);

        let input = &b"\"foo\tbar\""[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::LitExpr(Literal::StringLiteral(
            "foo\tbar".to_owned(),
        )))];

        assert_input_with_program(input, program);

        let input = &b"\"foo\\\"bar\""[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::LitExpr(Literal::StringLiteral(
            "foo\"bar".to_owned(),
        )))];

        assert_input_with_program(input, program);
    }

    #[test]
    fn arrays() {
        let input = &b"[1, 2 * 2, 3 + 3]"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::ArrayExpr(vec![
            Expr::LitExpr(Literal::IntLiteral(1)),
            Expr::InfixExpr(
                Infix::Multiply,
                Box::new(Expr::LitExpr(Literal::IntLiteral(2))),
                Box::new(Expr::LitExpr(Literal::IntLiteral(2))),
            ),
            Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::LitExpr(Literal::IntLiteral(3))),
                Box::new(Expr::LitExpr(Literal::IntLiteral(3))),
            ),
        ]))];

        assert_input_with_program(input, program);

        let input = &b"myArray[1 + 1]"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::IndexExpr {
            array: Box::new(Expr::IdentExpr(Ident("myArray".to_owned()))),
            index: Box::new(Expr::InfixExpr(
                Infix::Plus,
                Box::new(Expr::LitExpr(Literal::IntLiteral(1))),
                Box::new(Expr::LitExpr(Literal::IntLiteral(1))),
            )),
        })];

        assert_input_with_program(input, program);
    }

    #[test]
    fn array_precedence() {
        let input = "a * [1, 2, 3, 4][b * c] * d".as_bytes();

        let input2 = "((a * ([1, 2, 3, 4][b * c])) * d)".as_bytes();

        compare_inputs(input, input2);

        let input = "add(a * b[2], b[1], 2 * [1, 2][1])".as_bytes();

        let input2 = "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))".as_bytes();

        compare_inputs(input, input2);
    }

    #[test]
    fn hash() {
        let input = &b"{}"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::HashExpr(vec![]))];

        assert_input_with_program(input, program);

        let input = &b"{\"one\": 1, \"two\": 2, \"three\": 3}"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::HashExpr(vec![
            (
                Literal::StringLiteral("one".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(1)),
            ),
            (
                Literal::StringLiteral("two".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(2)),
            ),
            (
                Literal::StringLiteral("three".to_owned()),
                Expr::LitExpr(Literal::IntLiteral(3)),
            ),
        ]))];

        assert_input_with_program(input, program);

        let input = &b"{4: 1, 5: 2, 6: 3}"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::HashExpr(vec![
            (
                Literal::IntLiteral(4),
                Expr::LitExpr(Literal::IntLiteral(1)),
            ),
            (
                Literal::IntLiteral(5),
                Expr::LitExpr(Literal::IntLiteral(2)),
            ),
            (
                Literal::IntLiteral(6),
                Expr::LitExpr(Literal::IntLiteral(3)),
            ),
        ]))];

        assert_input_with_program(input, program);

        let input = &b"{true: 1, false: 2}"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::HashExpr(vec![
            (
                Literal::BoolLiteral(true),
                Expr::LitExpr(Literal::IntLiteral(1)),
            ),
            (
                Literal::BoolLiteral(false),
                Expr::LitExpr(Literal::IntLiteral(2)),
            ),
        ]))];

        assert_input_with_program(input, program);

        let input = &b"{\"one\": 0 + 1, \"two\": 10 - 8, \"three\": 15/5}"[..];

        let program: Program = vec![Stmt::ExprStmt(Expr::HashExpr(vec![
            (
                Literal::StringLiteral("one".to_owned()),
                Expr::InfixExpr(
                    Infix::Plus,
                    Box::new(Expr::LitExpr(Literal::IntLiteral(0))),
                    Box::new(Expr::LitExpr(Literal::IntLiteral(1))),
                ),
            ),
            (
                Literal::StringLiteral("two".to_owned()),
                Expr::InfixExpr(
                    Infix::Minus,
                    Box::new(Expr::LitExpr(Literal::IntLiteral(10))),
                    Box::new(Expr::LitExpr(Literal::IntLiteral(8))),
                ),
            ),
            (
                Literal::StringLiteral("three".to_owned()),
                Expr::InfixExpr(
                    Infix::Divide,
                    Box::new(Expr::LitExpr(Literal::IntLiteral(15))),
                    Box::new(Expr::LitExpr(Literal::IntLiteral(5))),
                ),
            ),
        ]))];

        assert_input_with_program(input, program);
    }
}
