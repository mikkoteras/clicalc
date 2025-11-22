use crate::errors::Error;
use crate::lexer::*;
use crate::parser::*;
use assert_approx_eq::assert_approx_eq;
use std::collections::HashMap;
use utility::*;

type EvaluationResult = Result<f64, Error>;

pub trait Evaluable {
	fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult;
}

impl Evaluable for Expression {
	fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		match &self {
			Expression::ParenExpr(e) => e.evaluate(variables),
			Expression::UnaryExpr(e) => e.evaluate(variables),
			Expression::BinaryExpr(e) => e.evaluate(variables),
			Expression::FunctionExpr(e) => e.evaluate(variables),
			Expression::VariableExpr(e) => e.evaluate(variables),
			Expression::LiteralExpr(e) => e.evaluate(variables)
		}
	}
}

impl Evaluable for ParenExpression {
	fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		self.expr.evaluate(variables)
	}
}

impl Evaluable for UnaryExpression {
	fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		let inner_result = self.expr.evaluate(variables)?;
		
		match &self.op {
			OperatorType::Plus => Ok(inner_result),
			OperatorType::Minus => Ok(-inner_result),
			_ => { panic!("Parser::UnaryExression::evaluate: parser is in an invalid state."); }
		}
	}
}

impl Evaluable for BinaryExpression {
	fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		let left_result = self.left.evaluate(variables)?;
		let right_result = self.right.evaluate(variables)?;
		
		match self.op {
			OperatorType::Plus => verify_result(left_result + right_result, "arithmetic overflow during addition"),
			OperatorType::Minus => verify_result(left_result - right_result, "arithmetic overflow during subtraction"),
			OperatorType::Times => verify_result(left_result * right_result, "arithmetic overflow during multiplication"),
			OperatorType::DividedBy => verify_result(left_result / right_result, "arithmetic overflow during division"),
			OperatorType::Power => verify_result(left_result.powf(right_result), "result of exponentiation is undefined"),
			_ => { panic!("BinaryExression::evaluate: parser is in an invalid state."); }
		}
	}
}

impl Evaluable for FunctionExpression {
    fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		let mut args: Vec<f64> = Vec::new();
		
		for	arg in &self.args {
			args.push(arg.evaluate(variables)?);
		}

        Ok(match &self.func {
			FunctionType::Abs => {
				require_fixed_args(args.len(), 1, "abs")?;
				args[0].abs()
			},
			FunctionType::ArcCos => {
				require_fixed_args(args.len(), 1, "arccos")?;
				verify_result(args[0].acos(), "arccos: argument must be between -1..1")?
			},
			FunctionType::ArcSin => {
				require_fixed_args(args.len(), 1, "arcsin")?;
				verify_result(args[0].asin(), "arcsin: argument must be between -1..1")?
			},
			FunctionType::ArcTan => {
				require_fixed_args(args.len(), 1, "arctan")?;
				args[0].atan()
			},
			FunctionType::Cos => {
				require_fixed_args(args.len(), 1, "cos")?;
				args[0].cos()
			},
			FunctionType::Exp => {
				require_fixed_args(args.len(), 1, "exp")?;
				verify_result(args[0].exp(), "exp: overflow")?
			},
			FunctionType::Ln => {
				require_fixed_args(args.len(), 1, "ln")?;
				verify_result(args[0].ln(), "ln: argument must be greater than zero")?
			},
			FunctionType::Log => {
				require_fixed_args(args.len(), 1, "log")?;
				verify_result(args[0].log10(), "log: argument must be greater than zero")?
			},
			FunctionType::Max => {
				require_min_args(args.len(), 2, "max")?;
				compute_max(args)
			},
			FunctionType::Min => {
				require_min_args(args.len(), 2, "min")?;
				compute_min(args)
			},
			FunctionType::Pow => {
				require_fixed_args(args.len(), 2, "pow")?;
				verify_result(args[0].powf(args[1]), "pow: the result is undefined")?
			},
			FunctionType::Sin => {
				require_fixed_args(args.len(), 1, "sin")?;
				args[0].sin()
			},
			FunctionType::Sqrt => {
				require_fixed_args(args.len(), 1, "sqrt")?;
				verify_result(args[0].sqrt(), "sqrt: argument must be nonnegative")?
			},
			FunctionType::Tan => {
				require_fixed_args(args.len(), 1, "tan")?;
				verify_result(args[0].tan(), "tan: result is undefined")?
			}
		})
    }
}

impl Evaluable for VariableExpression {

    fn evaluate(&self, variables: &HashMap<char, f64>) -> EvaluationResult {
		if let Some(val) = variables.get(&self.var).copied() {
			Ok(val)
		} else {
			error(&format!("variable {} is undefined", self.var))
		}
	}
}

impl Evaluable for LiteralExpression {
    fn evaluate(&self, _: &HashMap<char, f64>) -> EvaluationResult {
        Ok(self.val)
    }
}

mod utility {
	use crate::errors::Error;
	
	pub fn compute_min(args: Vec<f64>) -> f64 {
		let mut result = args[0];
		
		for a in &args[1..] {
			result = result.min(*a);
		}
		
		result
	}
	
	pub fn compute_max(args: Vec<f64>) -> f64 {
		let mut result = args[0];
		
		for a in &args[1..] {
			result = result.max(*a);
		}
		
		result
	}
	
	// Returns Err if the number of args is incorrect. The returned Ok() value is unusable.
	pub fn require_fixed_args(args_size: usize, required_size: usize, func_name: &str) -> Result<f64, Error> {
		if args_size == required_size {
			Ok(0.0)
		} else if required_size == 1 {
			error(&format!("{}: single argument required, got {}", func_name, args_size))
		} else {
			error(&format!("{}: {} arguments required, got {}", func_name, required_size, args_size))
		}
	}
	
	// Returns Err if the number of args is insufficient. The returned Ok() value is unusable.
	pub fn require_min_args(args_size: usize, required_min_size: usize, func_name: &str) -> Result<f64, Error> {
		if args_size >= required_min_size {
			Ok(0.0)
		} else {
			error(&format!("{}: at least {} arguments required, got {}", func_name, required_min_size, args_size))
		}
	}
	
	pub fn verify_result(result: f64, on_failure: &str) -> Result<f64, Error> {
		if result.is_finite() {
			Ok(result)
		} else {
			error(on_failure)
		}
	}
	
	pub fn error(description: &str) -> Result<f64, Error> {
		Err(Error::new(&format!("evaluation error: {}.", description)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_addition() {
		assert_approx_eq!(run_single_expression("2 + 6"), 8.0);
		assert_approx_eq!(run_single_expression("-2 + 6"), 4.0);
		assert_approx_eq!(run_single_expression("-2 + -6"), -8.0);
	}
	
	#[test]
	fn test_subtraction() {
		assert_approx_eq!(run_single_expression("2 - 6"), -4.0);
		assert_approx_eq!(run_single_expression("-2 - -6"), 4.0);
		assert_approx_eq!(run_single_expression("8 - -8"), 16.0);
	}
	
	#[test]
	fn test_multiplication() {
		assert_approx_eq!(run_single_expression("2 * 6"), 12.0);
		assert_approx_eq!(run_single_expression("2 * -6"), -12.0);
		assert_approx_eq!(run_single_expression("-2 * 6"), -12.0);
		assert_approx_eq!(run_single_expression("-2 * -6"), 12.0);
	}
	
	#[test]
	fn test_division() {
		assert_approx_eq!(run_single_expression("2 / 4"), 0.5);
		assert_approx_eq!(run_single_expression("20 / -4"), -5.0);
		assert_approx_eq!(run_single_expression("-2 / -1"), 2.0);
		expect_expression_to_fail("-2 / 0");
	}
	
	#[test]
	fn test_power() {
		assert_approx_eq!(run_single_expression("3 ^ 4"), 81.0);
		assert_approx_eq!(run_single_expression("9 ^ .5"), 3.0);
		assert_approx_eq!(run_single_expression("2^-2"), 0.25);
		expect_expression_to_fail("-1^0.5");
	}

	#[test]
	fn test_abs() {
		assert_approx_eq!(run_single_expression("abs(-3)"), 3.0);
	}
	
	#[test]
	fn test_arccos() {
	}
	
	#[test]
	fn test_arcsin() {
	}
	
	#[test]
	fn test_arctan() {
	}
	
	#[test]
	fn test_cos() {
		assert_approx_eq!(run_single_expression("cos(0.0)"), 1.0);
		assert_approx_eq!(run_single_expression("cos(0.5 * 3.14159265)"), 0.0);
		assert_approx_eq!(run_single_expression("cos(5.0 / 6.0 * 3.14159265) + sqrt(3) / 2"), 0.0);
		expect_expression_to_fail("cos(1.0, 2.0)");
	}
	
	#[test]
	fn test_exp() {
		assert_approx_eq!(run_single_expression("exp(2)"), 7.3890561);
		assert_approx_eq!(run_single_expression("exp(-2)"), 0.135335283);
		expect_expression_to_fail("exp(1.0, 1.0)");
	}
	
	#[test]
	fn test_ln() {
		assert_approx_eq!(run_single_expression("ln(2.718281828)"), 1.0);
		assert_approx_eq!(run_single_expression("ln(1.6487212707)"), 0.5);
		assert_approx_eq!(run_single_expression("ln(exp(sqrt(3))) - sqrt(3)"), 0.0);
		expect_expression_to_fail("ln(14.0, 1.0)");
		expect_expression_to_fail("ln(0.0)");
		expect_expression_to_fail("ln(-10.0)");
		expect_expression_to_fail("ln(1.0, 1.0)");
	}
	
	#[test]
	fn test_log() {
		assert_approx_eq!(run_single_expression("log(100)"), 2.0);
		assert_approx_eq!(run_single_expression("log(0.0100)"), -2.0);
		expect_expression_to_fail("log(14.0, 1.0)");
		expect_expression_to_fail("log(0.0)");
		expect_expression_to_fail("log(-10.0)");
	}
	
	#[test]
	fn test_max() {
		assert_approx_eq!(run_single_expression("max(0, 2)"), 2.0);
		assert_approx_eq!(run_single_expression("max(-1, -10, -2)"), -1.0);
		assert_approx_eq!(run_single_expression("max(sqrt(2), -1, 2^2)"), 4.0);
		expect_expression_to_fail("max(1.0)");
	}
	
	#[test]
	fn test_min() {
		assert_approx_eq!(run_single_expression("min(0, 2)"), 0.0);
		assert_approx_eq!(run_single_expression("min(-1, 0, 2)"), -1.0);
		assert_approx_eq!(run_single_expression("min(sqrt(5), -1, 2^2)"), -1.0);
		expect_expression_to_fail("min(1.0)");
	}
	
	#[test]
	fn test_pow() {
		assert_approx_eq!(run_single_expression("pow(16, 2)"), 256.0);
		assert_approx_eq!(run_single_expression("pow(2.5, 4)"), 39.0625);
		assert_approx_eq!(run_single_expression("pow(10, -3)"), 0.001);
		expect_expression_to_fail("pow(1.0)");
		expect_expression_to_fail("pow(0, -1)");
	}
		
	#[test]
	fn test_sin() {
		assert_approx_eq!(run_single_expression("sin(1.5 * 3.14159265)"), -1.0);
		assert_approx_eq!(run_single_expression("sin(2.0 / 12.0 * 3.14159265)"), 0.5);
		assert_approx_eq!(run_single_expression("sin(2.0 / 12.0 * 3.14159265)"), 0.5);
		expect_expression_to_fail("sin(1.0, 1.0)");
	}

	#[test]
	fn test_sqrt() {
		assert_approx_eq!(run_single_expression("sqrt(25)"), 5.0);
		assert_approx_eq!(run_single_expression("sqrt(.25)"), 0.5);
		expect_expression_to_fail("sqrt(-1.0)");
		expect_expression_to_fail("sqrt(1.0, 1.0)");
	}

	#[test]
	fn test_tan() {
		assert_approx_eq!(run_single_expression("tan(0)"), 0.0);
		assert_approx_eq!(run_single_expression("tan(3.1415926536 / 4)"), 1.0);
	}

	#[test]
	fn test_evaluation_order() {
		assert_approx_eq!(run_single_expression("(-8 - -7) - (-4 / -2)"), (-8.0 - -7.0) - (-4.0 / -2.0));
		assert_approx_eq!(run_single_expression("4*10^3+3*10^2+2*10^1+1*10^0"), 4321.0);
		assert_approx_eq!(run_single_expression("6/2(1+2)"), 9.0);
	}
	
	fn run_single_expression(line: &str) -> f64 {
		let mut parser = Parser::new(&line);
		let program = parser.parse().expect("expression doesn't parse!");
		
		match program {
			Program::Expr(expr) => {
				let variables = HashMap::<char, f64>::new(); // Not actually used
				expr.evaluate(&variables).expect("expression doesn't evaluate!")
			}
			_ => { panic!("not an expression!"); }
		}
	}
	
	fn expect_expression_to_fail(line: &str) {
		let mut parser = Parser::new(&line);
		let program = parser.parse().expect("expression doesn't parse!");
		
		match program {
			Program::Expr(expr) => {
				let variables = HashMap::<char, f64>::new(); // Not actually used
				
				match expr.evaluate(&variables) {
					Ok(_) => { panic!("expression should not evaluate!"); },
					_ => {}
				}
			}
			_ => { panic!("not an expression!"); }
		}
	}
}
