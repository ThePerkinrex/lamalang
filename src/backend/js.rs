use std::path::PathBuf;

use crate::{ast::Expr, error::Return, options::CodegenOptions, span::Span};

pub struct Codegen;

impl super::Backend for Codegen {
    fn gen_code(&self, expr: Span<Expr>, options: CodegenOptions) -> Return<()>{
        if options.lib {
            // Generate all items
        }else{
            // Generate only what is use from run
        }
        Ok(())
    }
}