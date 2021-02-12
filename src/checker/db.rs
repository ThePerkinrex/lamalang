use hashbrown::HashMap;

use crate::{ast::{Impl, TraitDef, TraitImpl}, span::Span};

pub type ModulePath = Vec<String>;

#[derive(Debug, Default)]
pub struct TypeDB {
	children_dbs: HashMap<String, TypeDBNode>
}

impl TypeDB {
	pub fn new_root() -> Self {
		Default::default()
	}
}

#[derive(Debug)]
pub enum TypeDBNode {
	DB(TypeDB),
	Reference(ModulePath),
	Type(Type),
	Trait(TraitDef)
}

#[derive(Debug)]
pub struct Type {
	generics: Vec<Span<String>>,
	impls: Vec<Impl>,
	trait_impls: Vec<(ModulePath, TraitImpl)>
}

impl Type {
	pub fn add_impl(&mut self, i: Impl) {
		self.impls.push(i)
	}

	pub fn add_trait_impl(&mut self, path: ModulePath, i: TraitImpl) {
		self.trait_impls.push((path, i))
	}
}