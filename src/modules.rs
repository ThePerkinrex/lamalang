use std::collections::HashMap;

use crate::{ast::AstModule, fs::File};

pub struct ModuleTree {
	externlibs: HashMap<String, ModuleTree>,
	root: Module
}

pub struct Module {
	ast: AstModule,
	children: HashMap<String, (bool, Module)>,
	allow_builtins: bool
}

pub fn build_tree(ast: (String, File, AstModule), allow_builtins: bool) -> Module {
	todo!()
}