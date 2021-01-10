use std::fmt::Display;

use pest::Parser;

use crate::{
	ast::{Block, Expr, Literal, Statement},
	fs::Fs,
	span::{BoxedSpan, File, Span},
};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct IdentParser;

pub fn parse(file: File, fs: &Fs) -> Span<Expr> {
	let s = fs.load_file(&file);
	let pairs = IdentParser::parse(Rule::calculation, &s).unwrap_or_else(|e| panic!("{}", e));
	// tree(pairs.clone(), "");

		{
			let pairs = IdentParser::parse(
				Rule::module,
				r#"
				fn hi< a, c >(a1: a) wherea: b, c: d {
					a + b;
					!a()() + b
				}
				trait Add<Other> {
					type Target;

					fn add(self: Self, other: Other) -> Target;
				}
				impl Add<number> for number {
					type Target = number;
					fn add(self: Self, other: number) -> Target {
						INTRINSIC_ADD_NUM
					}
				}
				"#,
			)
			.unwrap_or_else(|e| panic!("{}", e));
			tree(pairs.clone(), "");
		}

	eval_expr(pairs, &file).map(|x| *x)
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

fn eval_expr(pairs: pest::iterators::Pairs<Rule>, file: &File) -> BoxedSpan<Expr> {
	PREC_CLIMBER.climb(
		pairs,
		|pair: pest::iterators::Pair<Rule>| match pair.as_rule() {
			Rule::expr => eval_expr(pair.into_inner(), file),
			Rule::term => parse_term(pair.into_inner(), file),
			x => unreachable!("Unexpected rule: {:?} {:?}", x, pair.as_str()),
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

fn parse_term(pairs: pest::iterators::Pairs<Rule>, file: &File) -> BoxedSpan<Expr> {
	let mut unary_operators = vec![];
	let mut calls = vec![];
	let mut middle = None;
	for pair in pairs {
		match pair.as_rule() {
			Rule::unary => unary_operators.push(pair.into_inner().next().unwrap()),
			Rule::fn_call => calls.push(pair),
			Rule::value => middle = Some(parse_value(pair.into_inner(), file)),
			Rule::expr => middle = Some(eval_expr(pair.into_inner(), &file)),
			x => unreachable!("Unexpected rule in term: {:?} {:?}", x, pair.as_str()),
		}
	}
	let mut middle = middle.unwrap();
	for call in calls {
		let mut args = vec![];
		let span = call.as_span();
		for pair in call.into_inner() {
			args.push(eval_expr(pair.into_inner(), file))
		}
		middle = BoxedSpan::boxed(span, file.clone(), Expr::FnCall(middle, args));
	}
	for op in unary_operators {
		middle = match op.as_rule() {
			Rule::not => BoxedSpan::boxed(op.as_span(), file.clone(), Expr::Not(middle)),
			x => unreachable!("Unexpected rule in unary: {:?} {:?}", x, op.as_str()),
		}
	}
	middle
}

fn parse_value(mut pairs: pest::iterators::Pairs<Rule>, file: &File) -> BoxedSpan<Expr> {
	let pair = pairs.next().unwrap();
	BoxedSpan::boxed(
		pair.as_span(),
		file.clone(),
		match pair.as_rule() {
			Rule::num => Expr::Literal(Literal::Number(pair.as_str().parse::<f64>().unwrap())),
			Rule::string_content => {
				let s = pair.as_str();
				Expr::Literal(Literal::String(s.to_string()))
			}
			Rule::if_statement => {
				let mut inner = pair.into_inner();
				let condition = eval_expr(inner.next().unwrap().into_inner(), &file);
				let block = parse_block(inner.next().unwrap().into_inner(), &file);
				let mut elseif_clauses = vec![];
				let mut else_clause = None;
				for clause in inner {
					match clause.as_rule() {
						Rule::elseif_clause => {
							let mut inner = clause.into_inner();
							let condition = eval_expr(inner.next().unwrap().into_inner(), &file);
							let block = parse_block(inner.next().unwrap().into_inner(), &file);
							elseif_clauses.push((condition, block))
						}
						Rule::else_clause => {
							let mut inner = clause.into_inner();
							let block = parse_block(inner.next().unwrap().into_inner(), &file);
							else_clause = Some(block);
						}
						_ => unreachable!(),
					}
				}
				Expr::If(condition, block, elseif_clauses, else_clause)
			}
			_ => unreachable!(),
		},
	)
}

fn parse_block(pairs: pest::iterators::Pairs<Rule>, file: &File) -> Block {
	let mut statements = vec![];
	for statement in pairs {
		match statement.as_rule() {
			Rule::statement => statements.push(
				eval_expr(statement.into_inner(), file).map(|x| Box::new(Statement::Returning(*x))),
			),
			Rule::non_returning_statement => statements.push(
				eval_expr(statement.into_inner(), file)
					.map(|x| Box::new(Statement::NonReturning(*x))),
			),
			_ => unreachable!(),
		}
	}
	statements
}

fn tree<T: Display>(pairs: pest::iterators::Pairs<Rule>, space: T) {
	for pair in pairs {
		println!("{}{:?} [{}]", space, pair.as_rule(), pair.as_str());
		tree(pair.into_inner(), format!("{}  ", space));
	}
}
