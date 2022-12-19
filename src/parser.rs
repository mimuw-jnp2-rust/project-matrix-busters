#![allow(dead_code)]

use std::collections::VecDeque;

use anyhow::{bail, Context};
use num_traits::{CheckedAdd, CheckedMul, CheckedSub};

use crate::environment::{Environment, Identifier, Type};
use crate::matrices::Matrix;
use crate::traits::{CheckedMulScl, MatrixNumber};

#[derive(Debug)]
enum Token {
    Integer(u64),
    Identifier(Identifier),
    BinaryOp(char),
    LeftBracket,
    RightBracket,
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
        } else if self.raw.starts_with(|c| "+-*/".contains(c)) {
            let op = self.raw.chars().next().unwrap();
            self.raw = &self.raw[1..];
            Ok(Some(Token::BinaryOp(op)))
        } else if self.raw.starts_with(|c: char| c.is_ascii_digit()) {
            let i = self
                .raw
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(self.raw.len());
            let num = &self.raw[..i];
            self.raw = &self.raw[i..];
            Ok(Some(Token::Integer(num.parse::<u64>()?)))
        } else {
            let i = self
                .raw
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(self.raw.len());
            let id = Identifier::new(&self.raw[..i])?;
            self.raw = &self.raw[i..];
            Ok(Some(Token::Identifier(id)))
        }
    }
}

#[derive(Debug)]
enum WorkingToken<T: MatrixNumber> {
    Type(Type<T>),
    BinaryOp(char),
    LeftBracket,
    RightBracket,
}

fn binary_op<T: MatrixNumber>(left: Type<T>, right: Type<T>, op: char) -> anyhow::Result<Type<T>> {
    let wrap_matrix = |opt: Option<Matrix<T>>| match opt {
        Some(m) => Ok(Type::Matrix(m)),
        None => Err(anyhow::anyhow!("Operation error!")),
    };

    let wrap_scalar = |opt: Option<T>| match opt {
        Some(val) => Ok(Type::Scalar(val)),
        None => Err(anyhow::anyhow!("Operation error!")),
    };

    match op {
        '+' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => wrap_matrix(l.checked_add(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => wrap_scalar(l.checked_add(&r)),
            _ => bail!("Adding scalar to matrix is not supported!"),
        },
        '-' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => wrap_matrix(l.checked_sub(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => wrap_scalar(l.checked_sub(&r)),
            _ => bail!("Substraction of scalar and matrix is not supported!"),
        },
        '*' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => wrap_matrix(l.checked_mul(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => wrap_scalar(l.checked_mul(&r)),
            (Type::Matrix(l), Type::Scalar(r)) => wrap_matrix(l.checked_mul_scl(&r)),
            (Type::Scalar(l), Type::Matrix(r)) => wrap_matrix(r.checked_mul_scl(&l)),
        },
        '/' => match (left, right) {
            (Type::Scalar(l), Type::Scalar(r)) => wrap_scalar(l.checked_div(&r)),
            (Type::Matrix(_), Type::Matrix(_)) => bail!("WTF dividing by matrix? You should use the `inv` function (not implemented yet, wait for it...)"),
            (Type::Matrix(_), Type::Scalar(_)) => bail!("Diving matrix by scalar is not supported yet..."),
            (Type::Scalar(_), Type::Matrix(_)) => bail!("Diving scalar by matrix does not make sense!"),
        }
        _ => unimplemented!(),
    }
}

/*
<digit>      ::= "0" | "1" | ... | "9"
<integer>    ::= <digit>+
<letter>     ::= "a" | "ą" | "b" | ... | "ż"
<identifier> ::= (<letter> | "_") (<letter> | <digit> | "_")*
<binary_op>  ::= "+" | "-" | "*" | "/"
<expr>       ::= <integer> | <identifier> | <expr> <binary_op> <expr>
               | "(" <expr> ")"
 */
pub fn parse_expression<T: MatrixNumber>(
    raw: &str,
    env: &Environment<T>,
) -> anyhow::Result<Type<T>> {
    let mut tokenizer = Tokenizer::new(raw);
    let mut operators: VecDeque<WorkingToken<T>> = VecDeque::new();
    let mut outputs: VecDeque<WorkingToken<T>> = VecDeque::new();

    fn precedence(c: &char) -> u8 {
        match c {
            '+' | '-' => 0,
            '*' | '/' => 1,
            _ => unreachable!(),
        }
    }

    while let Some(token) = tokenizer.next_token()? {
        match token {
            Token::Integer(num) => outputs.push_back(WorkingToken::Type(Type::Scalar(
                T::from_u64(num).context("Number conversion failed!")?,
            ))),
            Token::Identifier(id) => outputs.push_back(WorkingToken::Type(
                env.get(&id).context("Undefined identifier!")?.clone(),
            )),
            Token::LeftBracket => operators.push_front(WorkingToken::LeftBracket),
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
            }
            Token::BinaryOp(op) => {
                while let Some(stack_token) = operators.pop_front() {
                    if let WorkingToken::BinaryOp(stack_op) = stack_token {
                        if precedence(&stack_op) >= precedence(&op) {
                            outputs.push_back(WorkingToken::BinaryOp(stack_op));
                        } else {
                            operators.push_front(WorkingToken::BinaryOp(stack_op));
                            break;
                        }
                    } else {
                        operators.push_front(stack_token);
                        break;
                    }
                }
                operators.push_front(WorkingToken::BinaryOp(op));
            }
        }
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
            _ => unreachable!(),
        }
    }

    val_stack.pop_front().context("Invalid expression!")
}

#[cfg(test)]
mod tests {
    use num_rational::Rational64;

    use crate::im;

    use super::*;

    #[test]
    fn test_expression_simple() {
        let mut env = Environment::new();
        env.insert(Identifier::new("a").unwrap(), Type::Scalar(2));
        env.insert(Identifier::new("b").unwrap(), Type::Scalar(3));
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
    }

    #[test]
    fn test_expression_identifiers() {
        let mut env = Environment::new();
        env.insert(
            Identifier::new("_i_love_rust_69").unwrap(),
            Type::Scalar(69),
        );
        env.insert(
            Identifier::new("_i_love_rust_42").unwrap(),
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
        env.insert(Identifier::new("a").unwrap(), Type::Scalar(2));
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

        env.insert(Identifier::new("A").unwrap(), Type::Matrix(a.clone()));
        env.insert(Identifier::new("B").unwrap(), Type::Matrix(b.clone()));
        env.insert(Identifier::new("Id_2").unwrap(), Type::Matrix(i2.clone()));

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A+A", a.clone() + a.clone());
        test_expr("A*B", a.clone() * b.clone());
        test_expr("A*B*Id_2", a.clone() * b.clone() * i2.clone());
        test_expr("Id_2-Id_2", i2.clone() - i2.clone());
    }

    #[test]
    fn test_expression_matrices_scalar() {
        let mut env = Environment::new();
        let a = im![1, 2, 3; 4, 5, 6];
        let b = im![1, 2; 3, 4; 5, 6];
        let i2 = im![1, 0; 0, 1];

        env.insert(Identifier::new("A").unwrap(), Type::Matrix(a.clone()));
        env.insert(Identifier::new("B").unwrap(), Type::Matrix(b.clone()));
        env.insert(Identifier::new("Id_2").unwrap(), Type::Matrix(i2.clone()));
        env.insert(Identifier::new("a").unwrap(), Type::Scalar(2));

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A+a*A", a.clone() + a.clone() * 2);
        test_expr("A*2*B", a.clone() * b.clone() * 2);
        test_expr("A*B*a*Id_2", a.clone() * b.clone() * i2.clone() * 2);
        test_expr("2*Id_2-Id_2", i2.clone() * 2 - i2.clone());
    }

    #[test]
    fn test_nested_multiplication() {
        let mut env = Environment::new();
        let fib = im![0, 1; 1, 1];

        env.insert(Identifier::new("A").unwrap(), Type::Matrix(fib.clone()));

        let test_expr = |raw, expected| {
            assert_eq!(parse_expression(raw, &env).unwrap(), Type::Matrix(expected))
        };

        test_expr("A*A*A*A*A*A*A*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*A*A*(A*A*A)*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*A*A*(A*(A*A))*A*A*A", im![34, 55; 55, 89]);
        test_expr("A*A*(A*A)*(A*(A*A))*A*A*A", im![34, 55; 55, 89]);
        // test_expr("(A*)A*(A(*A))*(A*(A*A))*A*A**A", im![34, 55; 55, 89]); TODO: This should not pass the test, but it does.
    }
}
