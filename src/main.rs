mod help;
mod lexer;
mod parser;
mod errors;
mod evaluation;
mod runner;

use std::io;
use crate::parser::*;
use crate::runner::*;

fn main() {
	println!("{}", env!("CARGO_PKG_NAME").to_string() + " " + env!("CARGO_PKG_VERSION"));
    println!("Type ? or help for usage, quit to exit.");
	
	let mut keep_going = true;
	let mut runner = Runner::new();
	
	while keep_going {
		let mut line = String::new();
		io::stdin()
			.read_line(&mut line)
			.expect("Input error!");

		let mut parser = Parser::new(&line);
		
		match parser.parse() {
			Ok(program) => {
				keep_going = runner.run(&program);
			},
			Err(e) => {
				println!("{}", e.description);
			}
		}
	}
}
