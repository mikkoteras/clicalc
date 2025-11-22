use crate::help::help;
use crate::lexer::*;
use crate::parser::*;
use crate::Program::*;
use crate::Statement::*;
use crate::evaluation::*;
use std::collections::HashMap;

pub struct Runner {
	variables: HashMap<char, f64>
}

impl Runner {
	pub fn new() -> Self {
		Self {
			variables: HashMap::<char, f64>::new()
		}
	}
	
	// Return false when it's time to exit.
	pub fn run(&mut self, program: &Program) -> bool {
		match program {
			Stmt(statement) => { 
				self.run_statement(statement)
			},
			Expr(expression) => {
				self.run_expression(expression)
			}
		}
	}

	fn run_statement(&mut self, statement: &Statement) -> bool {
		match statement {
			CommandStmt(stmt) => { self.run_command(stmt) },
			AssignmentStmt(stmt) => { self.run_assignment(stmt) }
		}
	}
	
	fn run_command(&self, statement: &CommandStatement) -> bool {
		match statement.command {
			CommandType::Help => {
				println!("{}", help());
			},
			CommandType::Quit => {
				return false;
			}
		}
		
		true
	}
	
	fn run_assignment(&mut self, assignment: &AssignmentStatement) -> bool {
		match assignment.expression.evaluate(&self.variables) {
			Ok(result) => {
				let var = assignment.variable.var;
				self.variables.insert(var, result);
				println!("{var} = {result}")
			}
			Err(e) => {
				println!("{}", e.description);
			}
		}
		
		true
	}
	
	fn run_expression(&self, expression: &Expression) -> bool {
		match expression.evaluate(&self.variables) {
			Ok(result) => {
				println!("{result}")
			}
			Err(e) => {
				println!("{}", e.description);
			}
		}
		
		true
	}
}
