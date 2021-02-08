use crate::span::{BoxedSpan, Span};

#[derive(Debug)]
pub struct AstModule {
	pub mods: Vec<Mod>,
	pub fns: Vec<FnDef>,
	pub trait_defs: Vec<TraitDef>,
	pub trait_impls: Vec<TraitImpl>,
	pub impls: Vec<Impl>
}

// impl AstModule {
// 	pub fn new(mods: Vec<Mod>, fns: Vec<FnDef>, trait_defs: Vec<TraitDef>) -> Self {
// 		Self { mods, fns, trait_defs }
// 	}
// }

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
pub struct FnSignatureDef {
	pub pub_kw: Option<Span<()>>,
	pub name: Span<String>,
	pub generics: Option<Generics>,
	pub where_clause: Option<WhereClause>,
	pub args: Vec<FnArg>,
	pub return_type: Span<Type>,
}

#[derive(Debug)]
pub struct TraitDef {
	pub pub_kw: Option<Span<()>>,
	pub name: Span<String>,
	pub generics: Option<Generics>,
	pub where_clause: Option<WhereClause>,
	pub fn_defs: Vec<FnDef>,
	pub fn_signatures: Vec<FnSignatureDef>,
	pub types: Vec<(Span<TypeInTrait>, Vec<Span<Trait>>)>
}

#[derive(Debug)]
pub struct TraitImpl {
	pub generics: Option<Generics>,
	pub trait_: Span<Trait>,
	pub type_: Span<Type>,
	pub where_clause: Option<WhereClause>,
	pub fn_defs: Vec<FnDef>,
	pub types: Vec<(Span<TypeInTrait>, Span<Type>)>
}

#[derive(Debug)]
pub struct Impl {
	pub generics: Option<Generics>,
	pub type_: Span<Type>,
	pub where_clause: Option<WhereClause>,
	pub fn_defs: Vec<FnDef>,
	pub types: Vec<(Span<TypeInTrait>, Span<Type>)>
}

impl Impl {
	pub fn to_trait_impl(self, trait_: Span<Trait>) -> TraitImpl {
		TraitImpl {
		    generics: self.generics,
		    trait_,
		    type_: self.type_,
		    where_clause: self.where_clause,
		    fn_defs: self.fn_defs,
		    types: self.types,

		}
	}
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

#[derive(Debug)]
pub struct TypeInTrait {
	pub name: Span<String>,
	pub generics: Vec<Span<String>>,
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
	Ident(Vec<Span<String>>),
}

#[derive(Debug)]
pub enum Literal {
	Float(f64),
	Int(i64),
	String(String),
}
