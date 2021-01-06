use crate::span::File;

#[derive(Default)]
pub struct Fs {
	repl: Vec<String>
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
			File::Path(p) => std::fs::read_to_string(p).unwrap()
		}
	}
}