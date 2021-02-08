use std::{collections::HashMap, path::{Path, PathBuf}};

use error::Return;
use fs::{File, Fs};

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

mod ast;
mod backend;
mod modules;
mod parser;

mod checker;

mod error;
mod fs;
mod span;

mod options;

use modules::ModuleTree;
use options::Options;
use structopt::StructOpt;

fn main() {
	std::process::exit(match wrapped_main() {
		Ok(()) => 0,
		Err(error::ReturnValue { value }) => value + 1,
	})
}

lazy_static! {
    static ref SYSROOT: &'static Path = Path::new("src/lamalib");
}


fn wrapped_main() -> Return<()> {
	let opt = options::Options::from_args();
	println!("{:?}", opt);
	let fs = Fs::default();
	if let Some(p) = opt.input_file {
		let p: PathBuf = p;
		let file = File::Path(p.clone());
		let m = parser::parse_module(file.clone(), &fs);
		let extern_libs = load_extern_libs(&fs, opt.no_std, opt.external)?;
		let module = modules::build_tree(
			&fs,
			&file,
			p.file_stem().unwrap().to_str().unwrap(),
			m,
			false,
		)?;
		println!("{}:\n\t{:?}\n\n", file, module);
		let module_tree = modules::ModuleTree::new(
			extern_libs,
			module,
		);
		// println!("{:?}", module_tree.root);
	// TODO pass module to codegen
	} else {
		// TODO Start repl
	}
	// let statement = fs.insert_repl_statement(
	// 	"!1 + 1 - 2 * (2 / 3 + \"HAHA\")(1 + 2, if 1 {} else if 2 {} else {})".to_string(),
	// );
	// let expr = parser::parse(statement, &fs);
	// let (codegen_opts, backend) = opt.into_codegen_options();
	// println!("> {}", backend.get_codegen().eval_expr(expr, codegen_opts)?);
	Ok(())
}

fn load_extern_libs(fs: &Fs, no_std: bool, extern_lib_list: Vec<(String, String)>) -> Return<HashMap<String, ModuleTree>> {
	let mut extern_libs = HashMap::default();
	load_extern_lib(&fs, &mut extern_libs, "core".into(), SYSROOT.join("core").join("lib.lama"), true)?;
	if !no_std {
		load_extern_lib(&fs, &mut extern_libs, "std".into(), SYSROOT.join("std").join("lib.lama"), true)?;
	}
	for (name, path) in extern_lib_list {
		load_extern_lib(&fs, &mut extern_libs, name, PathBuf::from(path), false)?;
	}
	Ok(extern_libs)
}

fn load_extern_lib(fs: &Fs, extern_libs: &mut HashMap<String, ModuleTree>, name: String, entry_point: PathBuf, allow_builtins: bool) -> Return<()> {
	let file = File::Path(entry_point);
	let ast = parser::parse_module(file.clone(), &fs);
	let module = modules::build_tree(
		fs,
		&file,
		&name,
		ast,
		allow_builtins,
	)?;
	println!("{}:\n\t{:?}\n\n", file, module);
	extern_libs.insert(name.clone(), modules::ModuleTree::new(
		Default::default(),
		module,
	));
	Ok(())
}
