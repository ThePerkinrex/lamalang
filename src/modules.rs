use std::{collections::HashMap, fmt::Debug};

use crate::{
	ast::AstModule,
	error::Return,
	fs::{File, Fs},
	span::Span,
};

pub struct ModuleTree {
	externlibs: HashMap<String, ModuleTree>,
	pub root: Module,
}

impl ModuleTree {
	pub fn new(externlibs: HashMap<String, ModuleTree>, root: Module) -> Self {
		Self { externlibs, root }
	}
}

pub struct Module {
	pub ast: AstModule,
	children: HashMap<String, (bool, Module)>,
	allow_builtins: bool,
}

impl Debug for Module {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} {{{}}}",
			if self.allow_builtins { " builtin" } else { "" },
			self.children
				.iter()
				.map(|(name, (public, m))| format!(
					"{}{}{:?}",
					if *public { "pub " } else { "" },
					name,
					m
				))
				.collect::<Vec<String>>()
				.join(", ")
		)
	}
}

pub fn build_tree(
	fs: &Fs,
	file: &File,
	module_name: &str,
	ast: AstModule,
	allow_builtins: bool,
) -> Return<Module> {
	let mut children = HashMap::new();
	for module in &ast.mods {
		let is_pub = module.pub_kw.is_some();
		let name = module.name.clone();
		let name_span_str = Span::new_ref(&name, |s| s.as_str());
		let name_str = name_span_str.as_ref();
		let module = fs.find_child(file, &module_name, name_span_str)?;
		let ast = crate::parser::parse_module(module, fs);
		let module = build_tree(fs, file, name_str, ast, allow_builtins)?;
		children.insert(name.into_inner(), (is_pub, module));
	}
	Ok(Module {
		children,
		ast,
		allow_builtins,
	})
}
