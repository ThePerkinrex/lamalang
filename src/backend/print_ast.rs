use crate::{
	ast::Expr,
	error::Return,
	options::CodegenOptions,
	span::Span,
};

pub struct Codegen;

impl super::Backend for Codegen {
	fn eval_expr(&self, expr: Span<Expr>, _options: CodegenOptions) -> Return<String> {
		println!("{:?}", expr);
		Ok(String::default())
	}

	fn codegen(&self, module: crate::modules::ModuleTree, _options: CodegenOptions) -> Return<String> {
        println!("{:?}", module.root.ast);
		Ok(String::default())
    }
}
