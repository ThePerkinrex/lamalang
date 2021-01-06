use std::path::PathBuf;

use crate::{ast::Expr, error::Return, options::CodegenOptions, span::Span};

pub mod js;
pub mod interpreter;

pub trait Backend {
	fn gen_code(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<()>;
}