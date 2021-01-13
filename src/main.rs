use error::Return;
use fs::{File, Fs};

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

mod ast;
mod backend;
mod parser;

mod error;
mod fs;
mod span;

mod options;

use structopt::StructOpt;

fn main() {
	std::process::exit(match wrapped_main() {
		Ok(()) => 0,
		Err(error::ReturnValue { value }) => value + 1,
	})
}

fn wrapped_main() -> Return<()> {
	let opt = options::Options::from_args();
	println!("{:?}", opt);
	let mut fs = Fs::default();
	if let Some(p) = opt.input_file {
		let m = parser::parse_module(File::Path(p.into()), &fs);
		println!("{:?}", m);
		// TODO pass module to codegen
	}else{
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
