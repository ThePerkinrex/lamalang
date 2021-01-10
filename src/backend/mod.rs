
use crate::{ast::Expr, error::Return, options::CodegenOptions, span::Span};

pub mod interpreter;
pub mod js;

pub trait Backend {
	fn eval_expr(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<String>;
}
