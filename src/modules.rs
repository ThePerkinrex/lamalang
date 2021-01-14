use std::collections::HashMap;

use crate::ast::AstModule;

pub struct ModuleTree {
	externlibs: HashMap<String, Module>,
	root: Module
}

pub struct Module {
	ast: AstModule,
	children: HashMap<String, (bool, Module)>
}