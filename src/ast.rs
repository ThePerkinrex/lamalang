use crate::span::{BoxedSpan, Span};

#[derive(Debug)]
pub struct AstModule {
	pub mods: Vec<Mod>,
	pub fns: Vec<FnDef>,
}

impl AstModule {
	pub fn new(mods: Vec<Mod>, fns: Vec<FnDef>) -> Self {
		Self { mods, fns }
	}
}

#[derive(Debug)]
pub struct Mod {
	pub pub_kw: Option<Span<()>>,
	pub name: Span<String>,
}

#[derive(Debug)]
pub struct FnDef {
	pub pub_kw: Option<Span<()>>,
	pub name: Span<String>,
	pub generics: Option<Generics>,
	pub where_clause: Option<WhereClause>,
	pub args: Vec<FnArg>,
	pub return_type: Span<Type>,
	pub body: Span<Block>,
}

#[derive(Debug)]
pub enum Type {
	Empty,
	// Array(Box<Self>),
	// Tuple(Vec<Span<Self>>),
	Other {
		name: Span<String>,
		generics: Vec<Span<Type>>,
	},
}

impl Default for Type {
	fn default() -> Self {
		Self::Empty
	}
}

#[derive(Debug)]
pub struct Trait {
	pub name: Span<String>,
	pub generics: Vec<Span<Type>>,
}

pub type Generics = Vec<Span<String>>;
pub type WhereClause = Vec<TypeBound>;
pub type TypeBound = (Span<Type>, Vec<Span<Trait>>);
pub type FnArg = (Span<String>, Span<Type>);

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
	Ident(String)
}

#[derive(Debug)]
pub enum Literal {
	Float(f64),
	Int(i64),
	String(String),
}
