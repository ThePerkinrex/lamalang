use std::{
	fmt::{Debug, Display},
	fs::read_to_string,
	path::PathBuf,
};

use crate::{
	error::{ErrorCode, Return},
	span::Span,
};

#[derive(Clone)]
pub enum File {
	Repl(usize),
	Path(PathBuf),
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
			File::Path(p) => read_to_string(p).unwrap(),
		}
	}

	pub fn find_child(&self, file: &File, current: &str, name: Span<&str>) -> Return<File> {
		let root = match file {
			File::Repl(_) => PathBuf::new(),
			File::Path(p) => p.parent().unwrap().to_path_buf(),
		};
		let name_str: &str = name.as_ref();
		// Has file `name.lama`?
		let single_file = root.join(format!("{}.lama", name));
		if single_file.exists() {
			Ok(File::Path(single_file))
		} else {
			let mod_file = root.join(name_str).join("mod.lama");
			if mod_file.exists() {
				Ok(File::Path(mod_file))
			} else {
				let folder_root = root.join(current);
				let single_file = folder_root.join(format!("{}.lama", name));
				if single_file.exists() {
					Ok(File::Path(single_file))
				} else {
					let mod_file = folder_root.join(name_str).join("mod.lama");
					if mod_file.exists() {
						Ok(File::Path(mod_file))
					} else {
						name.as_error(
							ErrorCode::ModuleNotFoundError,
							format!("Module `{}` not found relative to {}", name, current),
						)
						.display()?;
						unreachable!()
					}
				}
			}
		}
	}
}
