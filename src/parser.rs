use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use anyhow::{bail, Context};
use num_traits::checked_pow;

use crate::environment::{Environment, Identifier, Type};
use crate::matrices::Matrix;
use crate::traits::MatrixNumber;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Integer(u64),
    Identifier(Identifier),
    Operator(char),
    LeftBracket,
    RightBracket,
    LeftMatrixBracket,
    RightMatrixBracket,
    Semicolon,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "int {i}"),
            Token::Identifier(id) => write!(f, "id {}", id.to_string()),
            Token::Operator(op) => write!(f, "operator \"{op}\""),
            Token::LeftBracket => write!(f, "( bracket"),
            Token::RightBracket => write!(f, ") bracket"),
            Token::LeftMatrixBracket => write!(f, "[ bracket"),
            Token::RightMatrixBracket => write!(f, "] bracket"),
            Token::Semicolon => write!(f, "; semicolon"),
        }
    }
}

struct Tokenizer<'a> {
    raw: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(raw: &'a str) -> Self {
        Tokenizer { raw }
    }

    fn next_token(&mut self) -> anyhow::Result<Option<Token>> {
        self.raw = self.raw.trim_start();
        if self.raw.is_empty() {
            Ok(None)
        } else if self.raw.starts_with('(') {
            self.raw = &self.raw[1..];
            Ok(Some(Token::LeftBracket))
        } else if self.raw.starts_with(')') {
            self.raw = &self.raw[1..];
            Ok(Some(Token::RightBracket))
        } else if self.raw.starts_with('[') {
            self.raw = &self.raw[1..];
            Ok(Some(Token::LeftMatrixBracket))
        } else if self.raw.starts_with(']') {
            self.raw = &self.raw[1..];
            Ok(Some(Token::RightMatrixBracket))
        } else if self.raw.starts_with(';') {
            self.raw = &self.raw[1..];
            Ok(Some(Token::Semicolon))
        } else if self.raw.starts_with(|c| "+-*/^=".contains(c)) {
            let op = self.raw.chars().next().unwrap();
            self.raw = &self.raw[1..];
            Ok(Some(Token::Operator(op)))
        } else if self.raw.starts_with(|c: char| c.is_ascii_digit()) {
            let i = self
                .raw
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(self.raw.len());
            let num = &self.raw[..i];
            self.raw = &self.raw[i..];
            Ok(Some(Token::Integer(num.parse::<u64>()?)))
        } else if let Some(rest) = self.raw.strip_prefix(Identifier::RESULT) {
            self.raw = rest;
            Ok(Some(Token::Identifier(Identifier::result())))
        } else {
            let i = self
                .raw
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(self.raw.len());
            let id = Identifier::new(self.raw[..i].to_string())?;
            self.raw = &self.raw[i..];
            Ok(Some(Token::Identifier(id)))
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum WorkingToken<T: MatrixNumber> {
    Type(Type<T>),
    Function(Identifier),
    UnaryOp(char),
    BinaryOp(char),
    LeftBracket,
    RightBracket,
}

impl<T: MatrixNumber> Display for WorkingToken<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkingToken::Type(_) => write!(f, "value token"),
            WorkingToken::Function(_) => write!(f, "function token"),
            WorkingToken::UnaryOp(op) => write!(f, "unary operator \"{op}\""),
            WorkingToken::BinaryOp(op) => write!(f, "binary operator \"{op}\""),
            WorkingToken::LeftBracket => write!(f, "( bracket"),
            WorkingToken::RightBracket => write!(f, ") bracket"),
        }
    }
}

fn binary_op<T: MatrixNumber>(left: Type<T>, right: Type<T>, op: char) -> anyhow::Result<Type<T>> {
    match op {
        '+' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_result(l.checked_add(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_add(&r)),
            _ => bail!("Adding scalar to matrix is not supported!"),
        },
        '-' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_result(l.checked_sub(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_sub(&r)),
            _ => bail!("Substraction of scalar and matrix is not supported!"),
        },
        '*' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_result(l.checked_mul(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_mul(&r)),
            (Type::Matrix(l), Type::Scalar(r)) => Type::from_matrix_result(l.checked_mul_scl(&r)),
            (Type::Scalar(l), Type::Matrix(r)) => Type::from_matrix_result(r.checked_mul_scl(&l)),
        },
        '/' => match (left, right) {
            (Type::Scalar(l), Type::Scalar(r)) => {
                if !r.is_zero() {
                    Type::from_scalar_option(l.checked_div(&r))
                } else {
                    bail!("Division by zero!")
                }
            }
            (Type::Matrix(_), Type::Matrix(_)) => {
                bail!("WTF dividing by matrix? You should use the `inverse` function instead!")
            }
            (Type::Matrix(_), Type::Scalar(_)) => {
                bail!("Diving matrix by scalar is not supported yet...")
            }
            (Type::Scalar(_), Type::Matrix(_)) => {
                bail!("Diving scalar by matrix does not make sense!")
            }
        },
        '^' => {
            if let Type::Scalar(exp) = right {
                let exp = exp
                    .to_usize()
                    .context("Exponent should be a nonnegative integer.")?;
                match left {
                    Type::Scalar(base) => Type::from_scalar_option(checked_pow(base, exp)),
                    Type::Matrix(base) => Type::from_matrix_result(base.checked_pow(exp)),
                }
            } else {
                bail!("Exponent cannot be a matrix!");
            }
        }
        _ => unimplemented!(),
    }
}

fn unary_op<T: MatrixNumber>(arg: Type<T>, op: char) -> anyhow::Result<Type<T>> {
    match op {
        '+' => Ok(arg),
        '-' => match arg {
            Type::Matrix(m) => Type::from_matrix_result(m.checked_neg()),
            Type::Scalar(s) => Type::from_scalar_option(T::zero().checked_sub(&s)),
        },
        _ => unimplemented!(),
    }
}

fn parse_matrix_element<T: MatrixNumber>(raw: &str, env: &Environment<T>) -> anyhow::Result<T> {
    let raw = raw.trim();
    if raw.is_empty() {
        bail!("Empty matrix element");
    }

    // Handle unary operators
    if let Some(rest) = raw.strip_prefix('+') {
        return parse_matrix_element(rest.trim(), env);
    }

    if let Some(rest) = raw.strip_prefix('-') {
        let val = parse_matrix_element(rest.trim(), env)?;
        return T::zero()
            .checked_sub(&val)
            .context("Arithmetic operation resulted in overflow!");
    }

    // Try to parse as a number (including rational numbers if T supports it)
    if let Ok(val) = T::from_str(raw) {
        return Ok(val);
    }

    // Parse identifier
    if raw == Identifier::RESULT {
        let id = Identifier::result();
        if let Some(value) = env.get_value(&id) {
            return match value {
                Type::Scalar(s) => Ok(s.clone()),
                Type::Matrix(_) => bail!("Matrix elements cannot be matrices"),
            };
        } else {
            bail!("Undefined identifier: {}", raw);
        }
    } else if let Ok(id) = Identifier::new(raw.to_string()) {
        if let Some(value) = env.get_value(&id) {
            return match value {
                Type::Scalar(s) => Ok(s.clone()),
                Type::Matrix(_) => bail!("Matrix elements cannot be matrices"),
            };
        } else {
            bail!("Undefined identifier: {}", raw);
        }
    }

    bail!("Invalid matrix element: {}", raw);
}

fn parse_matrix<T: MatrixNumber>(raw: &str, env: &Environment<T>) -> anyhow::Result<Matrix<T>> {
    let raw = raw.trim();
    if !raw.starts_with('[') || !raw.ends_with(']') {
        bail!("Matrix must be enclosed in square brackets");
    }

    let content = &raw[1..raw.len() - 1].trim();
    if content.is_empty() {
        bail!("Empty matrix not allowed");
    }

    let rows: Vec<&str> = content.split(';').collect();
    let mut matrix_data: Vec<Vec<T>> = Vec::new();

    for (row_idx, row_str) in rows.iter().enumerate() {
        let row_str = row_str.trim();
        if row_str.is_empty() {
            bail!("Empty row at position {}", row_idx);
        }

        let elements: Vec<&str> = row_str.split_whitespace().collect();
        if elements.is_empty() {
            bail!("Row {} has no elements", row_idx);
        }

        let mut row_data: Vec<T> = Vec::new();
        for element_str in elements {
            let element = parse_matrix_element(element_str, env)?;
            row_data.push(element);
        }

        // Check that all rows have the same number of columns
        if !matrix_data.is_empty() && matrix_data[0].len() != row_data.len() {
            bail!("All rows must have the same number of elements. Row 0 has {} elements, row {} has {} elements",
                  matrix_data[0].len(), row_idx, row_data.len());
        }

        matrix_data.push(row_data);
    }

    Matrix::new(matrix_data)
}

/*
<digit>      ::= "0" | "1" | ... | "9"
<integer>    ::= <digit>+
<letter>     ::= "a" | "ą" | "b" | ... | "ż"
<identifier> ::= (<letter> | "_") (<letter> | <digit> | "_")* | "$"
<unary_op>   ::= "+" | "-"
<binary_op>  ::= "+" | "-" | "*" | "/"
<matrix_elem>::= <integer> | <identifier> | <unary_op> <matrix_elem>
<matrix_row> ::= <matrix_elem> (" " <matrix_elem>)*
<matrix>     ::= "[" <matrix_row> (";" <matrix_row>)* "]"
<expr>       ::= <integer> | <identifier> | <matrix> | <expr> <binary_op> <expr>
               | "(" <expr> ")" | <unary_op> <expr> | <identifier> "(" <expr> ")"
 */
pub fn parse_expression<T: MatrixNumber>(
    raw: &str,
    env: &Environment<T>,
) -> anyhow::Result<Type<T>> {
    let raw = raw.trim();

    // Check if the entire expression is just a matrix (starts with '[' and ends with ']' with balanced brackets)
    if raw.starts_with('[') {
        let mut bracket_count = 0;
        let mut found_end = false;
        for (i, ch) in raw.char_indices() {
            match ch {
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        // If we've closed all brackets and we're at the end, it's a pure matrix
                        if i == raw.len() - 1 {
                            found_end = true;
                        }
                        break;
                    }
                }
                _ => {}
            }
        }

        if found_end && bracket_count == 0 {
            let matrix = parse_matrix(raw, env)?;
            return Ok(Type::Matrix(matrix));
        }
    }

    let mut tokenizer = Tokenizer::new(raw);
    let mut operators: VecDeque<WorkingToken<T>> = VecDeque::new();
    let mut outputs: VecDeque<WorkingToken<T>> = VecDeque::new();
    let mut prev_token = None;

    fn precedence(c: &char) -> u8 {
        match c {
            '+' | '-' => 0,
            '*' | '/' => 1,
            '^' => 2,
            _ => unreachable!(),
        }
    }

    fn validate_neighbours<T: MatrixNumber>(
        previous: &Option<&WorkingToken<T>>,
        current: &Token,
    ) -> bool {
        match current {
            Token::Integer(_) | Token::Identifier(_) | Token::LeftBracket => matches!(
                previous,
                None | Some(WorkingToken::LeftBracket)
                    | Some(WorkingToken::BinaryOp(_))
                    | Some(WorkingToken::UnaryOp(_))
                    | Some(WorkingToken::Function(_))
            ),
            Token::Operator(_) => matches!(
                previous,
                None | Some(WorkingToken::RightBracket)
                    | Some(WorkingToken::Type(_))
                    | Some(WorkingToken::BinaryOp(_))
                    | Some(WorkingToken::LeftBracket)
            ),
            Token::RightBracket => matches!(
                previous,
                Some(WorkingToken::RightBracket) | Some(WorkingToken::Type(_))
            ),
            // Matrix tokens should not appear in regular expressions since we handle them at a higher level
            Token::LeftMatrixBracket | Token::RightMatrixBracket | Token::Semicolon => false,
        }
    }

    while let Some(token) = tokenizer.next_token()? {
        if !validate_neighbours(&prev_token, &token) {
            match prev_token {
                Some(prev_token) => {
                    bail!("Invalid expression! The {token} cannot follow {prev_token}")
                }
                None => bail!("Invalid expression! The {token} cannot be the first token!"),
            }
        }

        prev_token = match &token {
            Token::Integer(num) => {
                outputs.push_back(WorkingToken::Type(Type::Scalar(
                    T::from_u64(*num).context(format!(
                        "Number conversion failed! {num:?} cannot be parsed into {:?}",
                        std::any::type_name::<T>()
                    ))?,
                )));
                outputs.back()
            }
            Token::Identifier(id) => {
                if let Some(value) = env.get_value(id) {
                    outputs.push_back(WorkingToken::Type(value.clone()));
                    outputs.back()
                } else if env.get_function(id).is_some() {
                    operators.push_front(WorkingToken::Function(id.clone()));
                    operators.front()
                } else {
                    bail!(
                        "Undefined identifier! Object \"{}\" is unknown.",
                        id.to_string()
                    )
                }
            }
            Token::LeftBracket => {
                operators.push_front(WorkingToken::LeftBracket);
                operators.front()
            }
            Token::RightBracket => {
                let mut left_found = false;
                while let Some(op) = operators.pop_front() {
                    if matches!(op, WorkingToken::LeftBracket) {
                        left_found = true;
                        break;
                    }
                    outputs.push_back(op);
                }
                if !left_found {
                    bail!("Mismatched brackets!");
                }
                if let Some(op) = operators.pop_front() {
                    match op {
                        WorkingToken::UnaryOp(_) | WorkingToken::Function(_) => {
                            outputs.push_back(op)
                        }
                        _ => operators.push_front(op),
                    }
                }
                Some(&WorkingToken::RightBracket)
            }
            Token::Operator(op)
                if matches!(
                    prev_token,
                    None | Some(WorkingToken::LeftBracket) | Some(WorkingToken::BinaryOp(_))
                ) =>
            {
                if "+-".contains(*op) {
                    operators.push_front(WorkingToken::UnaryOp(*op));
                    operators.front()
                } else {
                    bail!("Operator {op} cannot be used as a unary operator.")
                }
            }
            Token::Operator(op) if "+-*/^".contains(*op) => {
                while let Some(stack_token) = operators.pop_front() {
                    if let WorkingToken::BinaryOp(stack_op) = stack_token {
                        if precedence(&stack_op) >= precedence(op) {
                            outputs.push_back(WorkingToken::BinaryOp(stack_op));
                        } else {
                            operators.push_front(WorkingToken::BinaryOp(stack_op));
                            break;
                        }
                    } else if let WorkingToken::UnaryOp(stack_op) = stack_token {
                        outputs.push_back(WorkingToken::UnaryOp(stack_op))
                    } else {
                        operators.push_front(stack_token);
                        break;
                    }
                }
                operators.push_front(WorkingToken::BinaryOp(*op));
                operators.front()
            }
            Token::Operator(_) => bail!("Assignment is not allowed in expressions!"),
            Token::LeftMatrixBracket => bail!(
                "Matrix brackets should not appear in expressions! Use matrix syntax: [1 2; 3 4]"
            ),
            Token::RightMatrixBracket => bail!(
                "Matrix brackets should not appear in expressions! Use matrix syntax: [1 2; 3 4]"
            ),
            Token::Semicolon => bail!("Semicolons should only appear in matrix syntax: [1 2; 3 4]"),
        };
    }

    while let Some(token) = operators.pop_front() {
        if matches!(token, WorkingToken::LeftBracket) {
            bail!("Mismatched brackets!");
        }
        outputs.push_back(token);
    }

    let mut val_stack: VecDeque<Type<T>> = VecDeque::new();
    while let Some(token) = outputs.pop_front() {
        match token {
            WorkingToken::Type(value) => val_stack.push_front(value),
            WorkingToken::BinaryOp(op) => {
                let right = val_stack.pop_front().context("Invalid expression!")?;
                let left = val_stack.pop_front().context("Invalid expression!")?;
                val_stack.push_front(binary_op(left, right, op)?)
            }
            WorkingToken::UnaryOp(op) => {
                let arg = val_stack.pop_front().context("Invalid expression!")?;
                val_stack.push_front(unary_op(arg, op)?);
            }
            WorkingToken::Function(id) => {
                let arg = val_stack.pop_front().context("Invalid expression!")?;
                val_stack.push_front(env.get_function(&id).unwrap()(arg)?);
            }
            _ => unreachable!(),
        }
    }

    val_stack.pop_front().context("Invalid expression!")
}

/*
<inst> ::= <identifier> = <expr> | <expr>
 */
pub fn parse_instruction<T: MatrixNumber>(
    raw: &str,
    env: &mut Environment<T>,
) -> anyhow::Result<Identifier> {
    let mut tokenizer = Tokenizer::new(raw);
    if let Some(Token::Identifier(id)) = tokenizer.next_token()? {
        if tokenizer.next_token()? == Some(Token::Operator('=')) {
            let value = parse_expression(tokenizer.raw, env)?;
            env.insert(id.clone(), value);
            return Ok(id);
        }
    }

    let value = parse_expression(raw, env)?;
    env.insert(Identifier::result(), value);
    Ok(Identifier::result())
}

#[cfg(test)]
mod tests {
    use crate::matrices::Matrix;
    use num_rational::Rational64;

    use crate::im;

    use super::*;

    #[test]
    fn test_expression_simple() {
        let mut env = Environment::new();
        env.insert(Identifier::new("a".to_string()).unwrap(), Type::Scalar(2));
        env.insert(Identifier::new("b".to_string()).unwrap(), Type::Scalar(3));
        assert_eq!(parse_expression("a+b*b", &env).unwrap(), Type::Scalar(11));
        assert_eq!(parse_expression("(a+b)*b", &env).unwrap(), Type::Scalar(15));
    }

    #[test]
    fn test_expression_numbers() {
        let env = Environment::new();

        let test_expr = |raw, a, b| {
            assert_eq!(
                parse_expression(raw, &env).unwrap(),
                Type::Scalar(Rational64::new(a, b))
            )
        };

        test_expr("2+2", 4, 1);
        test_expr("(2-6*9)/5", -52, 5);
        test_expr("2-6*9/5", -44, 5);
        test_expr("(2-6)*9/5", -36, 5);
        test_expr("(2-6*9/5)", -44, 5);
        test_expr("1/2^8", 1, 256);
    }

    #[test]
    fn test_expression_unary() {
        let mut env = Environment::new();
        env.insert(Identifier::new("x".to_string()).unwrap(), Type::Scalar(-5));
        let a = im![1, 2, 3; 4, 5, 6];
        env.insert(
            Identifier::new("A".to_string()).unwrap(),
            Type::Matrix(a.clone()),
        );

        let test_expr = |raw, expected| assert_eq!(parse_expression(raw, &env).unwrap(), expected);

        test_expr("-1", Type::Scalar(-1));
        test_expr("-x", Type::Scalar(5));
        test_expr("-1+3", Type::Scalar(2));
        test_expr("-(1+3)", Type::Scalar(-4));
        test_expr("2+(-2)", Type::Scalar(0));
        test_expr("1 - -1", Type::Scalar(2));
        test_expr("+1 + +1", Type::Scalar(2));
        test_expr("2 * -3", Type::Scalar(-6));
        test_expr("+2 * +3", Type::Scalar(6));

        test_expr("-A", Type::Matrix(a.checked_neg().unwrap()));
        test_expr("+A + -A - -A", Type::Matrix(a.clone()));
        test_expr("+A - +A + +A", Type::Matrix(a));
    }

    #[test]
    fn test_expression_identifiers() {
        let mut env = Environment::new();
        env.insert(
            Identifier::new("_i_love_rust_69".to_string()).unwrap(),
            Type::Scalar(69),
        );
        env.insert(
            Identifier::new("_i_love_rust_42".to_string()).unwrap(),
            Type::Scalar(42),
        );
        assert_eq!(
            parse_expression("_i_love_rust_69-_i_love_rust_42", &env).unwrap(),
            Type::Scalar(27)
        );
    }

    #[test]
    fn test_expression_whitespaces() {
        let mut env = Environment::new();
        env.insert(Identifier::new("a".to_string()).unwrap(), Type::Scalar(2));
        assert_eq!(
            parse_expression("a + 1+2\t*a", &env).unwrap(),
            Type::Scalar(7)
        );
    }

    #[test]
    fn test_expression_matrices() {
        let mut env = Environment::new();
        let a = im![1, 2, 3; 4, 5, 6];
        let b = im![1, 2; 3, 4; 5, 6];
        let i2 = im![1, 0; 0, 1];

        env.insert(
            Identifier::new("A".to_string()).unwrap(),
            Type::Matrix(a.clone()),
        );
        env.insert(
            Identifier::new("B".to_string()).unwrap(),
            Type::Matrix(b.clone()),
        );
        env.insert(
            Identifier::new("Id_2".to_string()).unwrap(),
            Type::Matrix(i2.clone()),
        );

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A+A", a.clone() + a.clone());
        test_expr("A*B", a.clone() * b.clone());
        test_expr("A*B*Id_2", a * b * i2.clone());
        test_expr("Id_2-Id_2", i2.clone() - i2);
    }

    #[test]
    fn test_expression_matrices_scalar() {
        let mut env = Environment::new();
        let a = im![1, 2, 3; 4, 5, 6];
        let b = im![1, 2; 3, 4; 5, 6];
        let c = im![2, 3; 0, -1];
        let i2 = im![1, 0; 0, 1];

        env.insert(
            Identifier::new("A".to_string()).unwrap(),
            Type::Matrix(a.clone()),
        );
        env.insert(
            Identifier::new("B".to_string()).unwrap(),
            Type::Matrix(b.clone()),
        );
        env.insert(
            Identifier::new("C".to_string()).unwrap(),
            Type::Matrix(c.clone()),
        );
        env.insert(
            Identifier::new("Id_2".to_string()).unwrap(),
            Type::Matrix(i2.clone()),
        );
        env.insert(Identifier::new("a".to_string()).unwrap(), Type::Scalar(2));

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A+a*A", a.clone() + a.clone() * 2);
        test_expr("A*2*B", a.clone() * b.clone() * 2);
        test_expr("A*B*a*Id_2", a * b * i2.clone() * 2);
        test_expr("2*Id_2-Id_2", i2.clone() * 2 - i2.clone());
        test_expr("C^0", i2);
        test_expr("C^1", c.clone());
        test_expr("C^2", c.clone() * c);
    }

    #[test]
    fn test_nested_multiplication() {
        let mut env = Environment::new();
        let fib = im![0, 1; 1, 1];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(fib));

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A^10", im![34, 55; 55, 89]);
        test_expr("A*A*A*A*A*A*A*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*A*A*(A*A*A)*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*A*A*(A*(A*A))*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*(A*A)*(A*(A*A))*A*A*A", im![34, 55; 55, 89]);
    }

    #[test]
    fn test_invalid_expressions() {
        let env = Environment::<i64>::new();

        let test_invalid_expr = |raw| assert!(matches!(parse_expression(raw, &env), Err(_)));

        test_invalid_expr("2**3");
        test_invalid_expr("2*(3*)5");
        test_invalid_expr("3*()4");
        test_invalid_expr("(2+(3-)3)");
        test_invalid_expr("()");
    }

    #[test]
    fn test_assignments_fibonacci() {
        let mut env = Environment::<i64>::new();

        let mut exec = |raw| parse_instruction(raw, &mut env).unwrap();

        exec("a = 0");
        exec("b = 1");
        for _ in 0..10 {
            exec("c = a + b");
            exec("a = b");
            exec("b = c");
        }

        assert_eq!(
            *env.get_value(&Identifier::new("b".to_string()).unwrap())
                .unwrap(),
            Type::<i64>::Scalar(89)
        );
    }

    #[test]
    fn test_expression_as_instruction() {
        let mut env = Environment::<i64>::new();

        let mut exec = |raw| parse_instruction(raw, &mut env).unwrap();

        exec("2 + 2");
        exec("a = $ ^ $");

        assert_eq!(
            *env.get_value(&Identifier::new("a".to_string()).unwrap())
                .unwrap(),
            Type::<i64>::Scalar(256)
        );
    }

    #[test]
    fn test_expression_functions() {
        let mut env = Environment::new();

        let a = im![1, 2, 3; 4, 5, 6];
        let at = im![1, 4; 2, 5; 3, 6];
        let b = im![1, 2; 3, 4];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(a));
        env.insert(
            Identifier::new("B".to_string()).unwrap(),
            Type::Matrix(b.clone()),
        );

        assert_eq!(
            parse_expression("transpose(A)", &env).unwrap(),
            Type::Matrix(at)
        );
        assert_eq!(
            parse_expression("identity(4)", &env).unwrap(),
            Type::Matrix(Matrix::identity(4))
        );
        assert_eq!(
            parse_expression("inverse(B)", &env).unwrap(),
            Type::Matrix(b.inverse().unwrap().result)
        );
    }

    #[test]
    fn test_nested_functions() {
        let mut env = Environment::new();

        let a = im![1, 2, 3; 4, 5, 6];
        let att = im![1, 2, 3; 4, 5, 6];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(a));

        assert_eq!(
            parse_expression("transpose(transpose(A))", &env).unwrap(),
            Type::Matrix(att)
        )
    }

    #[test]
    fn test_expr_with_function() {
        let mut env = Environment::new();

        let a = im![1, 2, 3; 4, 5, 6];
        let b = im![1, 2; 3, 4];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(a));
        env.insert(
            Identifier::new("B".to_string()).unwrap(),
            Type::Matrix(b.clone()),
        );

        assert_eq!(
            parse_expression("transpose(A) * B", &env).unwrap(),
            Type::Matrix(im![13, 18; 17, 24; 21, 30])
        );
    }

    #[test]
    fn test_expr_in_function() {
        let mut env = Environment::new();

        let a = im![1, 2, 3; 4, 5, 6];
        let i = Matrix::identity(2);
        let at = im![1, 4; 2, 5; 3, 6];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(a));
        env.insert(Identifier::new("I".to_string()).unwrap(), Type::Matrix(i));

        assert_eq!(
            parse_expression("transpose(I * A)", &env).unwrap(),
            Type::Matrix(at)
        );
    }

    #[test]
    fn test_complex_nested_function_with_expr() {
        let mut env = Environment::new();

        let a = im![1, 2, 3; 4, 5, 6];

        env.insert(Identifier::new("A".to_string()).unwrap(), Type::Matrix(a));

        assert_eq!(
            parse_expression(
                "transpose(transpose(identity(2137 - 2135 + 1 - 1 + (42 - 420) * 0) * A) + transpose(identity(2) * A))",
                &env
            ).unwrap(),
            Type::Matrix(im![2, 4, 6; 8, 10, 12])
        );
    }

    #[test]
    fn test_matrix_syntax_simple() {
        let env = Environment::<i64>::new();

        // Test simple 2x2 matrix
        assert_eq!(
            parse_expression("[1 4; 3 2]", &env).unwrap(),
            Type::Matrix(im![1, 4; 3, 2])
        );

        // Test 1x3 matrix (row vector)
        assert_eq!(
            parse_expression("[1 2 3]", &env).unwrap(),
            Type::Matrix(im![1, 2, 3])
        );

        // Test 3x1 matrix (column vector)
        assert_eq!(
            parse_expression("[1; 2; 3]", &env).unwrap(),
            Type::Matrix(im![1; 2; 3])
        );
    }

    #[test]
    fn test_matrix_syntax_rational() {
        let env = Environment::<Rational64>::new();

        // Test with rational numbers - create expected matrix manually
        let expected_matrix = Matrix::new_unsafe(vec![
            vec![Rational64::new(-1, 2), Rational64::new(5, 4)],
            vec![Rational64::new(1, 2), Rational64::new(-9, 2)],
        ]);

        assert_eq!(
            parse_expression("[-1/2 5/4; 1/2 -9/2]", &env).unwrap(),
            Type::Matrix(expected_matrix)
        );
    }

    #[test]
    fn test_matrix_syntax_with_variables() {
        let mut env = Environment::<Rational64>::new();
        env.insert(
            Identifier::new("a".to_string()).unwrap(),
            Type::Scalar(Rational64::new(2, 1)),
        );
        env.insert(
            Identifier::new("x".to_string()).unwrap(),
            Type::Scalar(Rational64::new(-3, 1)),
        );

        // Test with variables - create expected matrices manually
        let expected_matrix1 = Matrix::new_unsafe(vec![vec![
            Rational64::new(2, 1),
            Rational64::new(-3, 2),
            Rational64::new(-3, 1),
        ]]);

        assert_eq!(
            parse_expression("[a -3/2 x]", &env).unwrap(),
            Type::Matrix(expected_matrix1)
        );

        // Test with unary operators
        let expected_matrix2 = Matrix::new_unsafe(vec![
            vec![Rational64::new(2, 1), Rational64::new(3, 1)],
            vec![Rational64::new(-2, 1), Rational64::new(-3, 1)],
        ]);

        assert_eq!(
            parse_expression("[+a -x; -a +x]", &env).unwrap(),
            Type::Matrix(expected_matrix2)
        );
    }

    #[test]
    fn test_matrix_syntax_errors() {
        let env = Environment::<i64>::new();

        // Test empty matrix
        assert!(parse_expression("[]", &env).is_err());

        // Test mismatched row sizes
        assert!(parse_expression("[1 2; 3 4 5]", &env).is_err());

        // Test empty row
        assert!(parse_expression("[1 2; ; 3 4]", &env).is_err());

        // Test undefined variable
        assert!(parse_expression("[a b]", &env).is_err());
    }

    #[test]
    fn test_matrix_in_expressions() {
        let mut env = Environment::<i64>::new();

        // For now, test that individual matrices work
        env.insert(
            Identifier::new("A".to_string()).unwrap(),
            Type::Matrix(im![1, 2; 3, 4]),
        );
        env.insert(
            Identifier::new("B".to_string()).unwrap(),
            Type::Matrix(im![5, 6; 7, 8]),
        );

        // Test matrix variables in expressions
        assert_eq!(
            parse_expression("A + B", &env).unwrap(),
            Type::Matrix(im![6, 8; 10, 12])
        );

        assert_eq!(
            parse_expression("A * B", &env).unwrap(),
            Type::Matrix(im![19, 22; 43, 50])
        );
    }

    #[test]
    fn test_matrix_syntax_integration_examples() {
        let mut env = Environment::<Rational64>::new();

        // Test matrix examples from the issue description

        // Test 1: [1 4; 3 2]
        let result1 = parse_expression("[1 4; 3 2]", &env);
        assert!(result1.is_ok());
        println!("✅ [1 4; 3 2] = {}", result1.unwrap().to_string());

        // Test 2: [-1/2 5/4 5/2; 1/2 13/17 -9/2]
        let result2 = parse_expression("[-1/2 5/4 5/2; 1/2 13/17 -9/2]", &env);
        assert!(result2.is_ok());
        println!("✅ [-1/2 5/4 5/2; 1/2 13/17 -9/2] parsed successfully");

        // Test 3: [a -3/2 x] with variables
        env.insert(
            Identifier::new("a".to_string()).unwrap(),
            Type::Scalar(Rational64::new(1, 1)),
        );
        env.insert(
            Identifier::new("x".to_string()).unwrap(),
            Type::Scalar(Rational64::new(2, 1)),
        );

        let result3 = parse_expression("[a -3/2 x]", &env);
        assert!(result3.is_ok());
        println!("✅ [a -3/2 x] = {}", result3.unwrap().to_string());

        // Test 4: Single column matrix
        let result4 = parse_expression("[1; 2; 3]", &env);
        assert!(result4.is_ok());
        println!("✅ [1; 2; 3] = {}", result4.unwrap().to_string());
    }
}
