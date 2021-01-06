use error::Return;
use fs::Fs;

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

mod parser;
mod ast;
mod backend;

mod span;
mod error;
mod fs;

mod options;

use structopt::StructOpt;

fn main() {
    std::process::exit(match wrapped_main() {
        Ok(()) => 0,
        Err(error::ReturnValue {value}) => value + 1
    })
}

fn wrapped_main() -> Return<()> {
    let opt = options::Options::from_args();
    println!("{:?}", opt);
    let mut fs = Fs::default();
    let statement = fs.insert_repl_statement("1 + 1 - 2 * (2 / 3 + \"HAHA\")".to_string());
    let expr = parser::parse(statement, &fs);
    let (codegen_opts, backend) = opt.into_codegen_options();
    backend.get_codegen().gen_code(expr, codegen_opts)?;
    Ok(())
}
