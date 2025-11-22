use crate::errors::Error;
use crate::lexer::utility::*;
use std::collections::HashMap;
use std::fmt;

type LexerResult = Result<Token, Error>;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum OperatorType {
	Plus,
	Minus,
	Times,
	DividedBy,
	Power,
	LeftParen,
	RightParen,
	Comma,
	Assignment
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let spellings = HashMap::from([
			(OperatorType::Plus, '+'),
			(OperatorType::Minus, '-'),
			(OperatorType::Times, '*'),
			(OperatorType::DividedBy, '/'),
			(OperatorType::Power, '^'),
			(OperatorType::LeftParen, '('),
			(OperatorType::RightParen, ')'),
			(OperatorType::Comma, ','),
			(OperatorType::Assignment, '=')]);
        write!(f, "{}", spellings.get(self).unwrap())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FunctionType {
	Abs,
	ArcCos,
	ArcSin,
	ArcTan,
	Cos,
	Exp,
	Ln,
	Log,
	Max,
	Min,
	Pow,
	Sin,
	Sqrt,
	Tan
}

#[derive(Copy, Clone, PartialEq)]
pub enum CommandType {
	Help,
	Quit
}

#[derive(Copy, Clone, PartialEq)]
pub enum Token {
	Command(CommandType),
	Literal(f64),
	Operator(OperatorType),
	Variable(char),
	Function(FunctionType),
	Eol
}

pub struct Lexer<'a> {
	text: &'a str,
	current_token: Token
}

impl<'a> Lexer<'a> {
	pub fn new(s: &'a str) -> Self {
		Self {
			text: &s,
			current_token: Token::Eol
		}
	}

	pub fn current(&self) -> LexerResult {
		Ok(self.current_token)
	}

	pub fn get_next(&mut self) -> LexerResult {
		let leading_operator_symbols = "+-*/^(),="; // TODO make this an array
		self.skip_whitespace();
		
		if self.text.is_empty() {
			self.current_token = Token::Eol;
			return Ok(Token::Eol);
		}
		
		let first = self.text.chars().next().unwrap();
		
		if first.is_ascii_digit() || first == '.' {
			self.current_token = self.get_literal()?;
		} else if leading_operator_symbols.contains(first) {
			self.current_token = self.get_operator()?;
		} else if first.is_ascii_lowercase() {
			self.current_token = self.get_name()?;
		} else {
			return error(&format!("unrecognized character: {}", first));
		}
		
		Ok(self.current_token)
	}
	
	// Return the next token without moving to it.
	pub fn peek_next(&mut self) -> LexerResult {
		// Hacky but obvious: save current state, invoke get_next()
		// and restore state before returning.
		let saved_text = self.text;
		let saved_current_token = self.current_token;
		let next = self.get_next();
		self.text = saved_text;
		self.current_token = saved_current_token;
		next
	}
	
	fn get_literal(&mut self) -> LexerResult {
		use utility::scan_digits;
		let mut consumed = 0;
		consumed += scan_digits(&self.text);
		
		if self.text[consumed..].chars().next() == Some('.') {
			consumed += 1; // Decimal separator
			let decimals = scan_digits(&self.text[consumed..]);
			
			if decimals == 0 {
				return error("No digits following '.'");
			}
			
			consumed += decimals;
		}
		
		// Proceed to exponent segment if there is a separator and digits,
		// leave untouched otherwise.
		if self.text.chars().nth(consumed) == Some('e') && self.text.chars().nth(consumed + 1).is_some_and(|d| d.is_ascii_digit()) {
			consumed += 1; // Exponent separator
			consumed += scan_digits(&self.text[consumed..]);
		}			
		
		let val = &self.text[..consumed]
				.parse::<f64>()
				.expect("Lexer::get_literal(): number literal delimited incorrectly.");
		self.text = &self.text[consumed..];
		Ok(Token::Literal(*val))
	}
	
	fn get_operator(&mut self) -> LexerResult {
		if let Some(symbol) = self.text.chars().next() {
			let operators = [
				('+', OperatorType::Plus),
				('-', OperatorType::Minus),
				('*', OperatorType::Times),
				('/', OperatorType::DividedBy),
				('^', OperatorType::Power),
				('(', OperatorType::LeftParen),
				(')', OperatorType::RightParen),
				(',', OperatorType::Comma),
				('=', OperatorType::Assignment)
			];
		
			for (spelling, operator) in operators.iter() {
				if symbol == *spelling {
					self.text = &self.text[1..];
					return Ok(Token::Operator(*operator));
				}
			}
			
			error("Unexpected character")
		} else {
			// This is strange. Treat it as EOL.
			self.text = &self.text[self.text.len()..];
			Ok(Token::Eol)
		}
	}
	
	// Can return either a Variable, a Function or a Command,
	// depending on what can be matched.
	fn get_name(&mut self) -> LexerResult {
		let cmd_spellings = [
			("help", CommandType::Help),
			("quit", CommandType::Quit)
		];

		let func_spellings = [
			("abs", FunctionType::Abs),
			("arccos", FunctionType::ArcCos),
			("arcsin", FunctionType::ArcSin),
			("arctan", FunctionType::ArcTan),
			("cos", FunctionType::Cos),
			("exp", FunctionType::Exp),
			("ln", FunctionType::Ln),
			("log", FunctionType::Log),
			("max", FunctionType::Max),
			("min", FunctionType::Min),
			("pow", FunctionType::Pow),
			("sin", FunctionType::Sin),
			("sqrt", FunctionType::Sqrt),
			("tan", FunctionType::Tan)
		];
		
		for item in cmd_spellings.iter() {
			let (spelling, cmd) = item;
			
			if self.text.starts_with(spelling) {
				self.text = &self.text[spelling.len()..];
				return Ok(Token::Command(*cmd));
			}
		}
		
		for item in func_spellings.iter() {
			let (spelling, func) = item;
			
			if self.text.starts_with(spelling) {
				let len = spelling.len();
				self.text = &self.text[len..];
				return Ok(Token::Function(*func));
			}
		}
		
		let v = self.text.chars().next()
					.expect("Lexer::get_symbol(): lexer is in an invalid state.");
		self.text = &self.text[1..];
		Ok(Token::Variable(v))
	}
	
	fn skip_whitespace(&mut self) {
		while !self.text.is_empty() && self.text.chars().next().unwrap().is_whitespace() {
			self.text = &self.text[1..];
		}
	}
}

mod utility {
	use crate::errors::Error;
	
	// Return the number of digits at or following the current read position.
	pub fn scan_digits(segment: &str) -> usize {
		for (i, c) in segment.chars().enumerate() {
			if !c.is_ascii_digit() {
				return i;
			}
		}

		segment.len()
	}
	
	pub fn error(description: &str) -> super::LexerResult {
		Err(Error::new(&format!("Syntax error: {}.", description)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn integer_literal_is_tokenized() {
		let input = String::from("1325");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 1325.0);
	}

	#[test]
	fn decimal_literal_is_tokenized() {
		let input = String::from("13.25");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 13.25);
	}

	#[test]
	fn integer_and_exp_literal_is_tokenized() {
		let input = String::from("13e2");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 1300.0);
	}

	#[test]
	fn decimal_and_exp_literal_is_tokenized() {
		let input = String::from("13.25e2");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 1325.0);
	}

	#[test]
	fn decimal_with_leading_zero_is_tokenized() {
		let input = String::from(".1325");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 0.1325);
	}

	#[test]
	fn decimal_with_leading_zero_and_exponent_is_tokenized() {
		let input = String::from(".1325e2");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 13.25);
	}

	#[test]
	fn decimal_with_two_exponents_tokenizes_as_literal_and_variable() {
		let input = String::from("13.25e2e24");
		let mut lexer = Lexer::new(&input);
		assert_literal_token_with_value(lexer.get_next(), 1325.0);
		assert_variable_token_with_name(lexer.get_next(), 'e');
		assert_literal_token_with_value(lexer.get_next(), 24.0);
	}

	fn assert_literal_token_with_value(token: LexerResult, value: f64) {
		match token.expect("Syntax error") {
			Token::Literal(v) => { assert_eq!(v, value); },
			_ => { panic!(); }
		}
	}
	
	fn assert_variable_token_with_name(token: LexerResult, name: char) {
		match token.expect("Syntax error") {
			Token::Variable(c) => { assert_eq!(c, name); },
			_ => { panic!(); }
		}
	}
	
	// TODO: add more tests
}
