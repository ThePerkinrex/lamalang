use crate::span::{BoxedSpan, Span};

#[derive(Debug)]
pub enum Statement {
	Returning(Expr),
	NonReturning(Expr),
}

pub type Block = Vec<BoxedSpan<Statement>>;

#[derive(Debug)]
pub enum Expr {
	Literal(Literal),
	Add(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
	Sub(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
	Mul(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
	Div(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
	Pow(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
	Not(BoxedSpan<Expr>),
	FnCall(BoxedSpan<Expr>, Vec<BoxedSpan<Expr>>),
	If(
		BoxedSpan<Expr>,
		Block,
		Vec<(BoxedSpan<Expr>, Block)>,
		Option<Block>,
	),
}

#[derive(Debug)]
pub enum Literal {
	Number(f64),
	String(String),
}
