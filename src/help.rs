pub fn help() -> String {
		env!("CARGO_PKG_NAME").to_string() + " " + env!("CARGO_PKG_VERSION") +
		r#" is an interactive calculator that can be run in a terminal.
commands:
help            displays this help text.
quit            exits.
<var> = <expr>  evaluates <expr> and assigns the result to variable <var>.
<expr>          evaluates <expr> and displays the result.

<var> is single letter variable name, i.e., one of a..z.
<expr> is a mathematical expression, consisting of any or the following:

<number>          a number literal in the standard format:
                      [-]nnn[.nnn][e[-]nnn]
				      [-].nnn[e[-]nnn]
expr + expr       addition
expr - expr       subtraction
expr * expr       multiplication
expr / expr       division
expr ^ expr       exponentiation
-expr             unary negative expression
+expr             supported for completeness, but basically useless
(expr)            parentheses can be used to modify the order of evaluation
abs(expr)         absolute value
arccos(expr)      arc cosine
arcsin(expr)      arc sine
arctan(expr)      arc tangent
cos(expr)         cosine
exp(expr)         e to a power
ln(expr)          natural logarithm (base e)
log(expr)         logarithm (base 10)
max(e1, e2, ...)  maximum of arguments
min(e1, e2, ...)  minimum of arguments
pow(e1, e1)       e1 to power e2
sin(expr)         sine
sqrt(expr)        square root
tan(expr)         tangent
<var>             previously assigned value of a variable

Parentheses following a function name are mandatory as evaluation rules
would otherwise become confusing.

The multiplication sign '*' can be omitted when the right hand operand
is not a number.

Variables can only be referred to only after they have been assigned to
at least once. Variables can be assigned to multiple times, and can be
referred to in the right hand expression even when assigning to the
variable itself; i.e., the following is valid:
x = 10
x = x + 10

Standard evaluation order applies. Functions, parenthesized subexpressions
and unary expressions are evaluated first, then exponentiation, then
multiplication and division, and finally addition and subtraction.
The expression
6 / 2(1 + 2)
yields 9 (as it is the correct answer).

Infinities and undefined values are caught and cannot be assigned.

Example input:
a = 2
b = -5
c = 3
r = (-b + sqrt(b^2 - 4ac)) / (2a)
s = (-b - sqrt(b^2 - 4ac)) / (2a)"#
}
