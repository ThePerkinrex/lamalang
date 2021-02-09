use crate::{ast::Expr, error::Return, modules::ModuleTree, options::CodegenOptions, span::Span};

pub mod interpreter;
pub mod js;
pub mod print_ast;

pub trait Backend {
	fn eval_expr(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<String>;
	fn codegen(&self, module: ModuleTree, options: CodegenOptions) -> Return<String>;
}

impl<T: Backend> Backend for Box<T> {
    fn eval_expr(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<String> {
        self.as_ref().eval_expr(expr, options)
    }

    fn codegen(&self, module: ModuleTree, options: CodegenOptions) -> Return<String> {
        self.as_ref().codegen(module, options)
    }
}
