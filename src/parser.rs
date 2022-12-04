#![allow(dead_code)]

use std::collections::VecDeque;

use anyhow::{bail, Context};
use num_traits::{CheckedAdd, CheckedMul};

use crate::environment::{Environment, Identifier, Type};
use crate::matrices::Matrix;
use crate::traits::MatrixNumber;

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
        if self.raw.len() == 0 {
            Ok(None)
        } else if self.raw.starts_with("(") {
            self.raw = &self.raw[1..];
            Ok(Some(Token::LeftBracket))
        } else if self.raw.starts_with(")") {
            self.raw = &self.raw[1..];
            Ok(Some(Token::RightBracket))
        } else if self.raw.starts_with(|c| "+-*/".contains(c)) {
            let op = self.raw.chars().nth(0).unwrap(); // todo: get rid of unwrap
            self.raw = &self.raw[1..];
            Ok(Some(Token::BinaryOp(op)))
        } else if self.raw.starts_with(|c: char| c.is_digit(10)) {
            let i = self
                .raw
                .find(|c: char| !c.is_digit(10))
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
    Type(Type<T>), // TODO: stop copying stuff...
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
        '*' => match (left, right) {
            (Type::Matrix(l), Type::Matrix(r)) => wrap_matrix(l.checked_mul(&r)),
            (Type::Scalar(l), Type::Scalar(r)) => wrap_scalar(l.checked_mul(&r)),
            _ => bail!("Tudruj co z checked_mul macierzy i skalara???"),
        },
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
    use super::*;

    #[test]
    fn test_simple() {
        let mut env = Environment::new();
        env.insert(Identifier::new("a").unwrap(), Type::Scalar(2));
        env.insert(Identifier::new("b").unwrap(), Type::Scalar(3));
        assert_eq!(parse_expression("a+b*b", &env).unwrap(), Type::Scalar(11));
        assert_eq!(parse_expression("(a+b)*b", &env).unwrap(), Type::Scalar(15));
    }
}
