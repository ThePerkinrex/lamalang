use crate::{
	ast::Expr,
	error::{ErrorCode, NonLocatedError, Return},
	options::CodegenOptions,
	span::Span,
};

pub struct Codegen;

impl super::Backend for Codegen {
	fn eval_expr(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<String> {
		if options.lib {
			// Error (No main)
			NonLocatedError::new(
				ErrorCode::ErrorTest,
				"Can't interpret library as it has no entry point".to_string(),
			)
			.display()?;
			unreachable!()
		} else {
			// Execute it

			Ok(format!(""))
		}
	}
}
