use std::{collections::HashMap, fmt::{Display, Debug}, fs::read_to_string, path::PathBuf, rc::Rc};

#[derive(Clone)]
pub enum File {
	Repl(usize),
	Path(PathBuf)
}

impl Display for File {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
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
		println!("{:?}", file);
		match file {
			File::Repl(i) => self.repl[*i].clone(),
			File::Path(p) => read_to_string(p).unwrap()
		}
	}
}