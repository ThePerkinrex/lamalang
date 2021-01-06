use std::fmt::Display;

use pest::Parser;

use crate::{ast::{Expr, Literal}, fs::Fs, span::{BoxedSpan, File, Span}};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct IdentParser;

pub fn parse(file: File, fs: &Fs) -> Span<Expr> {
    let s = fs.load_file(&file);
    let pairs = IdentParser::parse(Rule::calculation, &s)
        .unwrap_or_else(|e| panic!("{}", e));
    // tree(pairs.clone(), "");

    as_ast(pairs, file).map(|x| *x)
}

lazy_static! {
    static ref PREC_CLIMBER: pest::prec_climber::PrecClimber<Rule> = {
        use pest::prec_climber::{Assoc::*, Operator, PrecClimber};
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right),
        ])
    };
}

fn as_ast(pairs: pest::iterators::Pairs<Rule>, file: File) -> BoxedSpan<Expr> {
    PREC_CLIMBER.climb(
        pairs,
        |pair: pest::iterators::Pair<Rule>| match pair.as_rule() {
            Rule::literal => {
                parse_literal(pair.into_inner(), file.clone()).map(|x| Box::new(Expr::Literal(*x)))
            }
            Rule::expr => as_ast(pair.into_inner(), file.clone()),
            _ => unreachable!(),
        },
        |lhs: BoxedSpan<Expr>, op: pest::iterators::Pair<Rule>, rhs: BoxedSpan<Expr>| match op
            .as_rule()
        {
            Rule::add => BoxedSpan::boxed_from_inner(
                &[lhs.as_range(), rhs.as_range()],
                file.clone(),
                Expr::Add(lhs, Span::new(op.as_span(), file.clone(), ()), rhs),
            ),
            Rule::subtract => BoxedSpan::boxed_from_inner(
                &[lhs.as_range(), rhs.as_range()],
                file.clone(),
                Expr::Sub(lhs, Span::new(op.as_span(), file.clone(), ()), rhs),
            ),
            Rule::multiply => BoxedSpan::boxed_from_inner(
                &[lhs.as_range(), rhs.as_range()],
                file.clone(),
                Expr::Mul(lhs, Span::new(op.as_span(), file.clone(), ()), rhs),
            ),
            Rule::divide => BoxedSpan::boxed_from_inner(
                &[lhs.as_range(), rhs.as_range()],
                file.clone(),
                Expr::Div(lhs, Span::new(op.as_span(), file.clone(), ()), rhs),
            ),
            Rule::power => BoxedSpan::boxed_from_inner(
                &[lhs.as_range(), rhs.as_range()],
                file.clone(),
                Expr::Pow(lhs, Span::new(op.as_span(), file.clone(), ()), rhs),
            ),
            _ => unreachable!(),
        },
    )
}

fn parse_literal(mut pairs: pest::iterators::Pairs<Rule>, file: File) -> BoxedSpan<Literal> {
    let pair = pairs.next().unwrap();
    BoxedSpan::boxed(
        pair.as_span(),
        file,
        match pair.as_rule() {
            Rule::num => Literal::Number(pair.as_str().parse::<f64>().unwrap()),
            Rule::string => {
                let s = pair.as_str();
                Literal::String(s[1..s.len() - 1].to_string())
            }
            _ => unreachable!(),
        },
    )
}

// fn tree<T: Display>(pairs: pest::iterators::Pairs<Rule>, space: T) {
//     for pair in pairs {
//         println!("{} {:?} [{}]", space, pair.as_rule(), pair.as_str());
//         tree(pair.into_inner(), format!("{} ", space));
//     }
// }
