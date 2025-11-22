use crate::lexer::*;
use crate::errors::Error;
use utility::error;

type ParseResult<T> = Result<T, Error>;

pub enum Expression {
	ParenExpr(Box<ParenExpression>),
	UnaryExpr(Box<UnaryExpression>),
	BinaryExpr(Box<BinaryExpression>),
	FunctionExpr(Box<FunctionExpression>),
	VariableExpr(Box<VariableExpression>),
	LiteralExpr(Box<LiteralExpression>)
}

pub struct ParenExpression {
	pub expr: Expression
}

impl ParenExpression {
	pub fn new(e: Expression) -> Self {
		Self {
			expr: e
		}
	}
}

pub struct UnaryExpression {
	pub op: OperatorType,
	pub expr: Expression
}

impl UnaryExpression {
	pub fn new(o: OperatorType, e: Expression) -> Self {
		Self {
			op: o,
			expr: e
		}
	}
}

pub struct BinaryExpression {
	pub op: OperatorType,
	pub left: Expression,
	pub right: Expression
}

impl BinaryExpression {
	pub fn new(o: OperatorType, l: Expression, r: Expression) -> Self {
		Self {
			op: o,
			left: l,
			right: r
		}
	}
}

pub struct FunctionExpression {
	pub func: FunctionType,
	pub args: Vec<Expression>
}

impl FunctionExpression {
    pub fn new(f: FunctionType, a: Vec<Expression>) -> Self {
		Self {
			func: f,
			args: a
		}
    }
}

pub struct VariableExpression {
	pub var: char
}

impl VariableExpression {
	pub fn new(c: char) -> Self {
		Self {
			var: c
		}
	}
}

pub struct LiteralExpression {
	pub val: f64
}

impl LiteralExpression {
    pub fn new(v: f64) -> Self {
		Self {
			val: v
		}
    }
}

pub enum Statement {
	CommandStmt(Box<CommandStatement>),
	AssignmentStmt(Box<AssignmentStatement>)
}

pub struct CommandStatement {
	pub command: CommandType
}

impl CommandStatement {
	pub fn new(cmd: CommandType) -> Self {
		Self {
			command: cmd
		}
	}
}

pub struct AssignmentStatement {
	pub variable: VariableExpression,
	pub expression: Expression
}

impl AssignmentStatement {
	pub fn new(var: VariableExpression, expr: Expression) -> Self {
		Self {
			variable: var,
			expression: expr
		}
	}
}

pub enum Program {
	Stmt(Box<Statement>),
	Expr(Box<Expression>)
}

pub struct Parser<'a> {
	text: &'a str,
	lexer: Lexer<'a>
}

impl<'a> Parser<'a> {
	pub fn new(s: &'a str) -> Self {
		Self {
			text: &s,
			lexer: Lexer::new(s)
		}
	}
	
	pub fn parse(&mut self) -> ParseResult<Program> {
		self.lexer = Lexer::new(self.text);
		self.lexer.get_next()?;
		self.parse_program()
	}
	
	fn parse_program(&mut self) -> ParseResult<Program> {
		match self.lexer.current()? {
			Token::Command(_) => {
				self.parse_command_program()
			},
			Token::Variable(_) => {
				// This is currently the sole reason why we need the
				// atrocious Lexer::peek_next(): we need to figure out
				// if we have an assignment or a simple expression, without
				// introducing more cumbersome syntax (like "let a = 1").
				// We could of course rewrite the lexer to tokenize the entire
				// input into a Token vector that can be looked up at will,
				// but for now, we can live with this.
				if let Token::Operator(op) = self.lexer.peek_next()? && op == OperatorType::Assignment {
					return self.parse_assignment_program();
				}
				else {
					return self.parse_expression_program();
				}
			}
			_ => {
				self.parse_expression_program()
			}
		}
	}
	
	fn parse_command_program(&mut self) -> ParseResult<Program> {
		if let Token::Command(cmd) = self.lexer.current()? {
			self.lexer.get_next()?;
			self.require_end_of_input()?;
			let stmt = CommandStatement::new(cmd);
			let stmt = Statement::CommandStmt(Box::new(stmt));
			let prog = Program::Stmt(Box::new(stmt));
			Ok(prog)		
		} else {
			panic!("Parser::parse_command_program(): logic error.");
		}
	}

	fn parse_assignment_program(&mut self) -> ParseResult<Program> {
		// These two were already vetted by the caller:
		let variable = self.lexer.current()?; // This is the variable
		self.lexer.get_next()?; // This is the assignment operator
		
		let var: char;
		
		match variable {
			Token::Variable(v) => { var = v },
			_ => { panic!("Parser::parse_assignment_program(): logic error."); }
		}
		
		let lhs = VariableExpression::new(var);
		self.lexer.get_next()?;
		let rhs = self.parse_expression()?;
		self.require_end_of_input()?;
		let stmt = AssignmentStatement::new(lhs, rhs);
		let stmt = Statement::AssignmentStmt(Box::new(stmt));
		Ok(Program::Stmt(Box::new(stmt)))
	}

	fn parse_expression_program(&mut self) -> ParseResult<Program> {
		let expr = self.parse_expression()?;
		self.require_end_of_input()?;
		Ok(Program::Expr(Box::new(expr)))
	}
	
	fn parse_expression(&mut self) -> ParseResult<Expression> {
		self.parse_additive_expression()
	}
	
	fn parse_additive_expression(&mut self) -> ParseResult<Expression> {
		let mut result = self.parse_multiplicative_expression()?;
		
		loop {
			match self.lexer.current()? {
				Token::Operator(op) if op == OperatorType::Plus || op == OperatorType::Minus => {
					self.lexer.get_next()?;
					let rhs = self.parse_multiplicative_expression()?;
					result = Expression::BinaryExpr(Box::new(BinaryExpression::new(op, result, rhs)));
				},
				_ => {
					break;
				}
			}
		}
		
		Ok(result)
	}
	
	fn parse_multiplicative_expression(&mut self) -> ParseResult<Expression> {
		let mut result = self.parse_power_expression()?;
		
		loop {
			match self.lexer.current()? {
				Token::Operator(op) if op == OperatorType::Times || op == OperatorType::DividedBy => {
					self.lexer.get_next()?;
					let rhs = self.parse_power_expression()?;
					result = Expression::BinaryExpr(Box::new(BinaryExpression::new(op, result, rhs)));
				},
				// Support constructs like a(b+c)
				Token::Operator(OperatorType::LeftParen) => {
					let rhs = self.parse_term()?;
					result = Expression::BinaryExpr(Box::new(BinaryExpression::new(OperatorType::Times, result, rhs)));
				}
				// Support constructs like "2x", "ax^2", "-3sqrt(...", etc
				Token::Variable(_) | Token::Function(_) => {
					let rhs = self.parse_power_expression()?;
					result = Expression::BinaryExpr(Box::new(BinaryExpression::new(OperatorType::Times, result, rhs)));
				},
				_ => {
					break;
				}
			}
		}
		
		Ok(result)
	}
	
	fn parse_power_expression(&mut self) -> ParseResult<Expression> {
		let mut result = self.parse_term()?;
		
		loop {
			if let Token::Operator(OperatorType::Power) = self.lexer.current()? {
				self.lexer.get_next()?;
				let rhs = self.parse_term()?;
				result = Expression::BinaryExpr(Box::new(BinaryExpression::new(OperatorType::Power, result, rhs)));
			} else {
				break;
			}
		}
		
		Ok(result)
	}

	fn parse_term(&mut self) -> ParseResult<Expression> {
		// Parse a top-precedence subexpression, i.e., a paren-expression, a unary operation,
		// a function invocation, a variable or a literal.
		match self.lexer.current()? {
			Token::Command(_) => { 
				error(&format!("unexpected command {}", "TODO"))
			},
			Token::Literal(val) => {
				self.lexer.get_next()?;
				Ok(Expression::LiteralExpr(Box::new(LiteralExpression::new(val))))
			},
			Token::Operator(op) => {
				if op == OperatorType::LeftParen {
					self.lexer.get_next()?;
					let expr = self.parse_expression()?;
					self.require_operator(OperatorType::RightParen)?;
					Ok(Expression::ParenExpr(Box::new(ParenExpression::new(expr))))
				} else if op == OperatorType::Plus || op == OperatorType::Minus {
					self.lexer.get_next()?;
					let expr = self.parse_term()?;
					Ok(Expression::UnaryExpr(Box::new(UnaryExpression::new(op, expr))))
				} else {
					error("")
				}
			},
			Token::Variable(var) => {
				self.lexer.get_next()?;
				Ok(Expression::VariableExpr(Box::new(VariableExpression::new(var))))
			},
			Token::Function(func) => {
				self.lexer.get_next()?;
				self.require_operator(OperatorType::LeftParen)?;
				let args = self.parse_expression_list()?;
				self.require_operator(OperatorType::RightParen)?;
				Ok(Expression::FunctionExpr(Box::new(FunctionExpression::new(func, args))))
			},
			Token::Eol => error("unexpected end of input."),
		}
	}
	
	fn parse_expression_list(&mut self) -> ParseResult<Vec<Expression>> {
		let mut args = Vec::<Expression>::new();
		let mut done = if let Token::Operator(OperatorType::RightParen) = self.lexer.current()? { true } else { false };
		
		while !done {
			args.push(self.parse_expression()?);
			
			match self.lexer.current()? {
				Token::Operator(o) if o == OperatorType::RightParen => {
					done = true; // Leave paren in place for the caller
 				},
				Token::Operator(o) if o == OperatorType::Comma => {
					self.lexer.get_next()?; // Consume
				},
				_ => {
					return error("either ')' or ',' must follow argument.");
				}
			}
		}
				
		Ok(args)
	}

	// Require and consume.
	fn require_operator(&mut self, t: OperatorType) -> ParseResult<Token> {
		let token = self.lexer.current()?;
		
		if let Token::Operator(op) = token && op == t {
			self.lexer.get_next()?;
			Ok(token)
		} else {
			error("")
		}
	}

	fn require_end_of_input(&mut self) -> ParseResult<Token> {
		let token = self.lexer.current()?;
		
		match token {
			Token::Eol => Ok(token),
			_ => {
				error("extra characters at the end of line.")
			}
		}
	}
}

mod utility {
	use crate::errors::Error;
	use super::ParseResult;
	
	pub fn error<T>(description: &str) -> ParseResult<T> {
		Err(Error::new(&format!("Parse error: {}", description)))
	}
}
