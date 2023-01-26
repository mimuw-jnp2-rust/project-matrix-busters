#![allow(dead_code)]

use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use anyhow::{bail, Context};
use num_traits::{checked_pow, CheckedAdd, CheckedMul, CheckedNeg, CheckedSub};

use crate::environment::{Environment, Identifier, Type};
use crate::traits::{CheckedMulScl, MatrixNumber};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Integer(u64),
    Identifier(Identifier),
    Operator(char),
    LeftBracket,
    RightBracket,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "int {}", i),
            Token::Identifier(id) => write!(f, "id {}", id.to_string()),
            Token::Operator(op) => write!(f, "operator \"{}\"", op),
            Token::LeftBracket => write!(f, "( bracket"),
            Token::RightBracket => write!(f, ") bracket"),
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkingToken<T: MatrixNumber> {
    Type(Type<T>),
    UnaryOp(char),
    BinaryOp(char),
    LeftBracket,
    RightBracket,
}

impl<T: MatrixNumber> Display for WorkingToken<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkingToken::Type(_) => write!(f, "value token"),
            WorkingToken::UnaryOp(op) => write!(f, "unary operator \"{}\"", op),
            WorkingToken::BinaryOp(op) => write!(f, "binary operator \"{}\"", op),
            WorkingToken::LeftBracket => write!(f, "( bracket"),
            WorkingToken::RightBracket => write!(f, ") bracket"),
        }
    }
}

fn binary_op<T: MatrixNumber>(left: Type<T>, right: Type<T>, op: char) -> anyhow::Result<Type<T>> {
    match op {
        '+' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_option(l.checked_add(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_add(&r)),
            _ => bail!("Adding scalar to matrix is not supported!"),
        },
        '-' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_option(l.checked_sub(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_sub(&r)),
            _ => bail!("Substraction of scalar and matrix is not supported!"),
        },
        '*' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => Type::from_matrix_option(l.checked_mul(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_mul(&r)),
            (Type::Matrix(l), Type::Scalar(r)) => Type::from_matrix_option(l.checked_mul_scl(&r)),
            (Type::Scalar(l), Type::Matrix(r)) => Type::from_matrix_option(r.checked_mul_scl(&l)),
        },
        '/' => match (left, right) {
            (Type::Scalar(l), Type::Scalar(r)) => Type::from_scalar_option(l.checked_div(&r)),
            (Type::Matrix(_), Type::Matrix(_)) => bail!("WTF dividing by matrix? You should use the `inv` function (not implemented yet, wait for it...)"),
            (Type::Matrix(_), Type::Scalar(_)) => bail!("Diving matrix by scalar is not supported yet..."),
            (Type::Scalar(_), Type::Matrix(_)) => bail!("Diving scalar by matrix does not make sense!"),
        },
        '^' => if let Type::Scalar(exp) = right {
            let exp = exp.to_usize().context("Exponent should be a nonnegative integer.")?;
            match left {
                Type::Scalar(base) => Type::from_scalar_option(checked_pow(base, exp)),
                Type::Matrix(base) => Type::from_matrix_option(base.checked_pow(exp).ok()),
            }
        } else {
            bail!("Exponent cannot be a matrix!");
        }
        _ => unimplemented!(),
    }
}

fn unary_op<T: MatrixNumber>(arg: Type<T>, op: char) -> anyhow::Result<Type<T>> {
    match op {
        '+' => Ok(arg),
        '-' => match arg {
            Type::Matrix(m) => Type::from_matrix_option(m.checked_neg()),
            Type::Scalar(s) => Type::from_scalar_option(T::zero().checked_sub(&s)),
        },
        _ => unimplemented!(),
    }
}

/*
<digit>      ::= "0" | "1" | ... | "9"
<integer>    ::= <digit>+
<letter>     ::= "a" | "ą" | "b" | ... | "ż"
<identifier> ::= (<letter> | "_") (<letter> | <digit> | "_")* | "$"
<unary_op>   ::= "+" | "-"
<binary_op>  ::= "+" | "-" | "*" | "/"
<expr>       ::= <integer> | <identifier> | <expr> <binary_op> <expr>
               | "(" <expr> ")" | <unary_op> <expr>
 */
pub fn parse_expression<T: MatrixNumber>(
    raw: &str,
    env: &Environment<T>,
) -> anyhow::Result<Type<T>> {
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
                outputs.push_back(WorkingToken::Type(
                    env.get(id)
                        .context(format!(
                            "Undefined identifier! Object \"{}\" is unknown.",
                            id.to_string()
                        ))?
                        .clone(),
                ));
                outputs.back()
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
                    if matches!(op, WorkingToken::UnaryOp(_)) {
                        outputs.push_back(op);
                    } else {
                        operators.push_front(op);
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
    if let Ok(value) = parse_expression(raw, env) {
        env.insert(Identifier::result(), value);
        return Ok(Identifier::result());
    }

    let mut tokenizer = Tokenizer::new(raw);
    if let Some(Token::Identifier(id)) = tokenizer.next_token()? {
        if tokenizer.next_token()? == Some(Token::Operator('=')) {
            let value = parse_expression(tokenizer.raw, env)?;
            env.insert(id.clone(), value);
            Ok(id)
        } else {
            bail!("Unrecognized instruction!")
        }
    } else {
        bail!("Invalid instruction!")
    }
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
            *env.get(&Identifier::new("b".to_string()).unwrap()).unwrap(),
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
            *env.get(&Identifier::new("a".to_string()).unwrap()).unwrap(),
            Type::<i64>::Scalar(256)
        );
    }
}
