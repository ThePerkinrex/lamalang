use std::{collections::HashMap, fmt::{Display, Debug}, fs::read_to_string, path::PathBuf, rc::Rc};

#[derive(Clone)]
pub enum File {
	Repl(usize),
	Module(ModulePath),
	Path(PathBuf)
}

impl Display for File {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Module(p) => write!(f, "{}", p),
			Self::Repl(i) => write!(f, "repl[{}]", i),
			Self::Path(p) => write!(f, "{}", p.display()),
		}
	}
}

impl Debug for File {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self, f)
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ModulePath {
	elements: Vec<String>,
}

impl ModulePath {
	pub fn new(repr: String) -> Self {
		Self {
			elements: repr.split("::").map(ToString::to_string).collect()
		}
	}

	pub fn pop_top(mut self) -> (String, Self) {
		(self.elements.remove(0), self)
	}
	
	pub fn len(&self) -> usize {
		self.elements.len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl Display for ModulePath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.elements.join("::"))
	}
}

impl Debug for ModulePath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}

#[derive(Default)]
pub struct Fs {
	repl: Vec<String>,
}

impl Fs {
	pub fn insert_repl_statement(&mut self, statement: String) -> File {
		let res = File::Repl(self.repl.len());
		self.repl.push(statement);
		res
	}

	pub fn load_file<'a>(&self, file: &File) -> String {
		match file {
			File::Repl(i) => self.repl[*i].clone(),
			File::Module(p) => todo!("resolve module, for external & internal modules"),
			File::Path(p) => read_to_string(p).unwrap()
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Module {
	modules: HashMap<String, (bool, Module)>,
	external: Rc<HashMap<String, Module>>,
	file: PathBuf,
}

impl Module {
	pub fn new_from_external(external: HashMap<String, Module>, file: PathBuf) -> Self {
		Self {
			external: Rc::new(external),
			file,
			..Default::default()
		}
	}

	pub fn add_child<'a>(&'a mut self, public: bool, name: String, file: PathBuf) -> &'a mut Self {
		self.modules.insert(name.clone(), (public, Self {
			external: self.external.clone(),
			file,
			..Default::default()
		}));
		let (_, m) = self.modules.get_mut(&name).unwrap();
		m
	}

	pub fn resolve(&self, path: ModulePath) -> Option<Self> {
		let (parent, path) = path.pop_top();
		if self.external.contains_key(&parent) {
			self.external.get(&parent).unwrap().internal_resolve(true, path).and_then(|(public, x)| if public {Some(x)} else {None})
		}else if self.modules.contains_key(&parent) {
			let (public, module) = self.modules.get(&parent).unwrap();
			module.internal_resolve(*public, path).and_then(|(public, x)| if public {Some(x)} else {None})
		}else{None}
	}

	fn internal_resolve(&self, public: bool, path: ModulePath) -> Option<(bool, Self)> {
		if path.is_empty() {
			return Some((public, self.clone()));
		}
		let (parent, path) = path.pop_top();
		if self.modules.contains_key(&parent) {
			let (public, module) = self.modules.get(&parent).unwrap();
			module.internal_resolve(*public, path)
		}else{None}
	}

	pub fn file(&self) -> &std::path::Path {
		&self.file
	}
}
