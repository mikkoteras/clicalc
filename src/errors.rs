use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Error {
	pub description: String
}

impl Error {
	pub fn new(s: &str) -> Self {
		Self {
			description: String::from(s)
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "Syntax error: {}", self.description)
	}
}
