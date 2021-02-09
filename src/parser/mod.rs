use std::fmt::Display;

use pest::{
	iterators::{Pair, Pairs},
	Parser,
};

use crate::{
	ast::{
		AstModule, Block, Expr, FnArg, FnDef, FnSignatureDef, Generics, Impl, Literal, Mod,
		Statement, Trait, TraitDef, Type, TypeInTrait, WhereClause,
	},
	fs::{File, Fs},
	span::{BoxedSpan, Span},
};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct LamaParser;

fn tree<T: Display>(pairs: Pairs<Rule>, space: T) {
	for pair in pairs {
		println!("{}{:?} [{}]", space, pair.as_rule(), pair.as_str());
		tree(pair.into_inner(), format!("{}  ", space));
	}
}

pub fn parse_module(file: File, fs: &Fs) -> AstModule {
	let s = fs.load_file(&file);
	let pairs = LamaParser::parse(Rule::module, &s).unwrap_or_else(|e| panic!("{}", e));
	// tree(pairs.clone(), "");
	let mut mod_items = Vec::new();
	let mut fn_items = Vec::new();
	let mut trait_defs = Vec::new();
	let mut trait_impls = Vec::new();
	let mut impls = Vec::new();
	for pair in pairs {
		match pair.as_rule() {
			Rule::EOI => (),
			Rule::fn_item => {
				let mut inner = pair.into_inner();
				let pub_kw = parse_pub(&mut inner, Rule::fn_kw, file.clone());
				let name = parse_name(&mut inner, file.clone());
				let (generics, next) = parse_def_generics(&mut inner, &file);
				let args = parse_fn_def_args(next.unwrap(), &file);
				let return_type = {
					let next = inner.next().unwrap();
					let span = next.as_span();
					next.into_inner()
						.next()
						.map(|x| parse_type(x, file.clone()))
						.unwrap_or(Span::new(span, file.clone(), Type::Empty))
				};
				let possible_where = inner.next().unwrap();
				let (where_clause, body) = if possible_where.as_rule() == Rule::where_clause {
					(
						Some(parse_where_clause(possible_where, &file)),
						inner.next().unwrap(),
					)
				} else {
					(None, possible_where)
				};
				let body = Span::new(
					body.as_span(),
					file.clone(),
					parse_block(body.into_inner(), &file),
				);
				fn_items.push(FnDef {
					pub_kw,
					name,
					generics,
					args,
					return_type,
					where_clause,
					body,
				})
			}
			Rule::mod_item => {
				let mut inner = pair.into_inner();
				let pub_kw = parse_pub(&mut inner, Rule::mod_kw, file.clone());
				let name = parse_name(&mut inner, file.clone());
				mod_items.push(Mod { pub_kw, name })
			}
			Rule::trait_item => {
				let mut inner = pair.into_inner();
				let pub_kw = parse_pub(&mut inner, Rule::trait_kw, file.clone());
				let name = parse_name(&mut inner, file.clone());
				let (generics, next) = parse_def_generics(&mut inner, &file);
				let possible_where = next.unwrap();
				let (where_clause, next) = if possible_where.as_rule() == Rule::where_clause {
					(
						Some(parse_where_clause(possible_where, &file)),
						inner.next().unwrap(),
					)
				} else {
					(None, possible_where)
				};
				let mut fn_signatures = Vec::new();
				let mut fn_defs = Vec::new();
				let mut types = Vec::new();

				parse_trait_item(next, &file, &mut types, &mut fn_signatures, &mut fn_defs);
				for pair in inner {
					parse_trait_item(pair, &file, &mut types, &mut fn_signatures, &mut fn_defs);
				}
				trait_defs.push(TraitDef {
					pub_kw,
					name,
					generics,
					where_clause,
					fn_defs,
					fn_signatures,
					types,
				});
				// println!("TRAITS: {:?}", trait_defs);
			}
			Rule::impl_trait_item => {
				let mut inner = pair.into_inner();
				inner.next().unwrap(); // Ignore impl kw
				let (generics, next) = parse_def_generics(&mut inner, &file);
				let trait_ = parse_trait(next.unwrap(), file.clone());
				inner.next().unwrap(); // Ignore for kw
				let type_ = parse_type(inner.next().unwrap(), file.clone());

				let where_clause = if inner.peek().unwrap().as_rule() == Rule::where_clause {
					Some(parse_where_clause(inner.next().unwrap(), &file))
				} else {
					None
				};
				let trait_impl = parse_impl_inner(inner, &file, type_, generics, where_clause)
					.to_trait_impl(trait_);
				trait_impls.push(trait_impl);
			}
			Rule::impl_item => {
				let mut inner = pair.into_inner();
				inner.next().unwrap(); // Ignore impl kw
				let (generics, next) = parse_def_generics(&mut inner, &file);
				// let trait_ = parse_trait(next.unwrap(), file.clone());
				// inner.next().unwrap(); // Ignore for kw
				let type_ = parse_type(next.unwrap(), file.clone());

				let where_clause = if inner.peek().unwrap().as_rule() == Rule::where_clause {
					Some(parse_where_clause(inner.next().unwrap(), &file))
				} else {
					None
				};
				let impl_ = parse_impl_inner(inner, &file, type_, generics, where_clause);
				impls.push(impl_);
			}
			x => unreachable!("Unknown item: {:?}", x),
		}
	}
	AstModule {
		mods: mod_items,
		fns: fn_items,
		trait_defs,
		trait_impls,
		impls,
	}
}

fn parse_impl_inner(
	pairs: Pairs<Rule>,
	file: &File,
	type_: Span<Type>,
	generics: Option<Generics>,
	where_clause: Option<WhereClause>,
) -> Impl {
	let mut fn_defs = Vec::new();
	let mut types = Vec::new();
	for item in pairs {
		// println!("{:?}", item.as_rule());
		match item.as_rule() {
			Rule::impl_type => {
				let mut inner = item.into_inner();
				inner.next().unwrap(); // Ignore the type_kw
				let type_def = {
					let inner = inner.next().unwrap();
					let span = inner.as_span();
					let mut inner = inner.into_inner();
					let name = parse_name(&mut inner, file.clone());
					let generics = inner
						.map(|pair| {
							Span::new(pair.as_span(), file.clone(), pair.as_str().to_string())
						})
						.collect();
					Span::new(span, file.clone(), TypeInTrait { name, generics })
				};
				let type_ = parse_type(inner.next().unwrap(), file.clone());
				types.push((type_def, type_))
			}
			Rule::impl_fn => {
				let mut inner = item.into_inner();
				let pub_kw = parse_pub(&mut inner, Rule::fn_kw, file.clone());
				let name = parse_name(&mut inner, file.clone());
				let (generics, next) = parse_def_generics(&mut inner, &file);
				let args = parse_fn_def_args(next.unwrap(), &file);
				let return_type = {
					let next = inner.next().unwrap();
					let span = next.as_span();
					next.into_inner()
						.next()
						.map(|x| parse_type(x, file.clone()))
						.unwrap_or(Span::new(span, file.clone(), Type::Empty))
				};
				let possible_where = inner.next().unwrap();
				let (where_clause, body) = if possible_where.as_rule() == Rule::where_clause {
					(
						Some(parse_where_clause(possible_where, &file)),
						inner.next().unwrap(),
					)
				} else {
					(None, possible_where)
				};
				let body = Span::new(
					body.as_span(),
					file.clone(),
					parse_block(body.into_inner(), &file),
				);
				fn_defs.push(FnDef {
					pub_kw,
					name,
					generics,
					args,
					return_type,
					where_clause,
					body,
				})
			}
			_ => unreachable!(),
		}
	}
	Impl {
		generics,
		type_,
		where_clause,
		fn_defs,
		types,
	}
}

fn parse_trait_item(
	pair: Pair<Rule>,
	file: &File,
	types: &mut Vec<(Span<TypeInTrait>, Vec<Span<Trait>>)>,
	fn_signatures: &mut Vec<FnSignatureDef>,
	fn_defs: &mut Vec<FnDef>,
) {
	match pair.as_rule() {
		Rule::trait_type => {
			let mut inner = pair.into_inner();
			inner.next().unwrap(); // Ignore the type_kw
			let type_def = {
				let inner = inner.next().unwrap();
				let span = inner.as_span();
				let mut inner = inner.into_inner();
				let name = parse_name(&mut inner, file.clone());
				let generics = inner
					.map(|pair| Span::new(pair.as_span(), file.clone(), pair.as_str().to_string()))
					.collect();
				Span::new(span, file.clone(), TypeInTrait { name, generics })
			};
			let traits = inner.map(|pair| parse_trait(pair, file.clone())).collect();
			types.push((type_def, traits))
		}
		Rule::trait_fn => {
			let mut inner = pair.into_inner();
			let pub_kw = parse_pub(&mut inner, Rule::fn_kw, file.clone());
			let name = parse_name(&mut inner, file.clone());
			let (generics, next) = parse_def_generics(&mut inner, &file);
			let args = parse_fn_def_args(next.unwrap(), &file);
			let return_type = {
				let next = inner.next().unwrap();
				let span = next.as_span();
				next.into_inner()
					.next()
					.map(|x| parse_type(x, file.clone()))
					.unwrap_or(Span::new(span, file.clone(), Type::Empty))
			};
			let (where_clause, body) = if let Some(possible_where) = inner.next() {
				if possible_where.as_rule() == Rule::where_clause {
					(
						Some(parse_where_clause(possible_where, &file)),
						inner.next(),
					)
				} else {
					(None, Some(possible_where))
				}
			} else {
				(None, None)
			};
			let body = body.map(|body| {
				Span::new(
					body.as_span(),
					file.clone(),
					parse_block(body.into_inner(), &file),
				)
			});

			if let Some(body) = body {
				fn_defs.push(FnDef {
					pub_kw,
					name,
					generics,
					args,
					return_type,
					where_clause,
					body,
				})
			} else {
				fn_signatures.push(FnSignatureDef {
					pub_kw,
					name,
					generics,
					where_clause,
					args,
					return_type,
				})
			}
		}
		_ => unreachable!(),
	}
}

/// Parse a possible pub keyword and ignore the next pair
fn parse_pub(inner: &mut Pairs<Rule>, next_rule: Rule, file: File) -> Option<Span<()>> {
	parse_maybe_rule_matching_next(inner, Rule::pub_kw, next_rule)
		.unwrap() // Assume there must be a next expression must match thanks to the PEG definition in grammar.pest
		.map(|pair| Span::new(pair.as_span(), file, ()))
}

/// Parse an ident (or any pair as a Span<String>)
fn parse_name(inner: &mut Pairs<Rule>, file: File) -> Span<String> {
	let name = inner.next().unwrap();
	Span::new(name.as_span(), file, name.as_str().to_string())
}

/// Parse a type
fn parse_type(typ: Pair<Rule>, file: File) -> Span<Type> {
	let span = typ.as_span();
	let mut inner = typ.into_inner();
	let first = inner.next().unwrap();
	let content = match first.as_rule() {
		Rule::ident =>
		/* Other type */
		{
			let name = Span::new(first.as_span(), file.clone(), first.as_str().to_string());
			let generics = if let Some(p) = inner.next() {
				parse_generics(p, &file)
			} else {
				Vec::new()
			};

			Type::Other { name, generics }
		}
		Rule::empty_type => Type::Empty,
		_ => unreachable!(),
	};
	Span::new(span, file, content)
}

/// Parse a trait
fn parse_trait(trait_: Pair<Rule>, file: File) -> Span<Trait> {
	let span = trait_.as_span();
	let mut inner = trait_.into_inner();
	let first = inner.next().unwrap();
	let content = match first.as_rule() {
		Rule::ident =>
		/* Other type */
		{
			let name = Span::new(first.as_span(), file.clone(), first.as_str().to_string());
			let generics = if let Some(p) = inner.next() {
				parse_generics(p, &file)
			} else {
				Vec::new()
			};

			Trait { name, generics }
		}
		_ => unreachable!(),
	};
	Span::new(span, file, content)
}

/// Parse definition generics
fn parse_def_generics<'a>(
	inner: &'a mut Pairs<Rule>,
	file: &File,
) -> (Option<Generics>, Option<Pair<'a, Rule>>) {
	let (generics, next) = parse_maybe_rule(inner, Rule::def_generics);
	(
		generics.map(|x| {
			let inner = x.into_inner();
			let mut generics = Vec::new();
			for pair in inner {
				generics.push(Span::new(
					pair.as_span(),
					file.clone(),
					pair.as_str().to_string(),
				))
			}
			generics
		}),
		next,
	)
}

/// Parse type generics
fn parse_generics(inner: Pair<Rule>, file: &File) -> Vec<Span<Type>> {
	let mut r = Vec::new();
	for pair in inner.into_inner() {
		r.push(parse_type(pair, file.clone()))
	}
	r
}

/// Parse type generics
fn parse_where_clause(inner: Pair<Rule>, file: &File) -> WhereClause {
	let mut r = Vec::new();
	for pair in inner.into_inner() {
		let mut inner = pair.into_inner();
		let type_ = parse_type(inner.next().unwrap(), file.clone());
		let mut traits = Vec::with_capacity(1);
		for trait_ in inner {
			traits.push(parse_trait(trait_, file.clone()));
		}
		r.push((type_, traits));
	}
	r
}

/// Parse function def arguments
fn parse_fn_def_args<'a>(arguments: Pair<Rule>, file: &File) -> Vec<FnArg> {
	let mut r = Vec::new();
	for argument in arguments.into_inner() {
		let mut inner = argument.into_inner();
		let name = parse_name(&mut inner, file.clone());
		let typ = parse_type(inner.next().unwrap(), file.clone());
		r.push((name, typ));
	}
	r
}
/// Parse a possible pair and ignore the next pair
fn parse_maybe_rule_matching_next<'a>(
	inner: &'a mut Pairs<Rule>,
	rule: Rule,
	next_rule: Rule,
) -> Option<Option<Pair<'a, Rule>>> {
	let (maybe, next) = parse_maybe_rule(inner, rule);
	if next.unwrap().as_rule() == next_rule {
		Some(maybe)
	} else {
		None
	}
}

/// Parse a possible pair and return the next one
fn parse_maybe_rule<'a>(
	inner: &'a mut Pairs<Rule>,
	rule: Rule,
) -> (Option<Pair<'a, Rule>>, Option<Pair<'a, Rule>>) {
	let possible_pub = inner.next().unwrap();
	match possible_pub.as_rule() {
		x if x == rule => (Some(possible_pub), inner.next()),
		_ => (None, Some(possible_pub)),
	}
}

// pub fn parse(file: File, fs: &Fs) -> Span<Expr> {
// 	let s = fs.load_file(&file);
// 	let pairs = LamaParser::parse(Rule::calculation, &s).unwrap_or_else(|e| panic!("{}", e));
// 	// tree(pairs.clone(), "");

// 	// {
// 	// 	let pairs = LamaParser::parse(
// 	// 		Rule::module,
// 	// 		r#"
// 	// 		fn hi< a, c >(a1: a) wherea: b, c: d {
// 	// 			a + b;
// 	// 			!a()() + b
// 	// 		}
// 	// 		trait Add<Other> {
// 	// 			type Target;

// 	// 			fn add(self: Self, other: Other) -> Target;
// 	// 		}
// 	// 		impl Add<number> for number {
// 	// 			type Target = number;
// 	// 			fn add(self: Self, other: number) -> Target {
// 	// 				INTRINSIC_ADD_NUM
// 	// 			}
// 	// 		}
// 	// 		"#,
// 	// 	)
// 	// 	.unwrap_or_else(|e| panic!("{}", e));
// 	// 	tree(pairs.clone(), "");
// 	// }

// 	eval_expr(pairs, &file).map(|x| *x)
// }

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
			Rule::int => Expr::Literal(Literal::Int(pair.as_str().parse().unwrap())),
			Rule::float => Expr::Literal(Literal::Float(pair.as_str().parse().unwrap())),
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
			Rule::ident => Expr::Ident(parse_path(pair, &file)),
			x => unreachable!("Unexpected value: {:?}", x),
		},
	)
}

fn parse_path(pair: Pair<Rule>, file: &File) -> Vec<Span<String>> {
	// println!("{}", pair.as_str());
	pair.into_inner()
		.map(|x| Span::new(x.as_span(), file.clone(), x.as_str().to_string()))
		.collect()
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
