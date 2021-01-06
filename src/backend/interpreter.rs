use crate::{ast::Expr, error::{ErrorCode, NonLocatedError, Return}, options::CodegenOptions, span::Span};

pub struct Codegen;

impl super::Backend for Codegen {
    fn gen_code(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<()>{
        if options.lib {
            // Error (No main)
            NonLocatedError::new(ErrorCode::ErrorTest, "Can't interpret library as it has no entry point".to_string()).display()?;
        }else{
            // Execute it
        }
        Ok(())
    }
}