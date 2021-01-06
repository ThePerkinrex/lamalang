use crate::span::{BoxedSpan, Span};

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    Add(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
    Sub(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
    Mul(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
    Div(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>),
    Pow(BoxedSpan<Expr>, Span<()>, BoxedSpan<Expr>)
}

#[derive(Debug)]
pub enum Literal{
	Number(f64),
	String(String),
}