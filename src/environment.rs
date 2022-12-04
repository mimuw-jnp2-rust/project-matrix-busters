#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::bail;

use crate::{matrices::Matrix, traits::MatrixNumber};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    id: String,
}

impl Identifier {
    pub fn new(id: &str) -> anyhow::Result<Self> {
        if Self::is_valid(id) {
            Ok(Self { id: id.to_string() })
        } else {
            bail!("Invalid identifier.")
        }
    }

    pub fn is_valid(id: &str) -> bool {
        id.chars().all(|c| c.is_alphanumeric() || c == '_')
            && id.starts_with(|c: char| c.is_alphabetic() || c == '_')
    }

    pub fn get(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<T: MatrixNumber> {
    Scalar(T),
    Matrix(Matrix<T>),
}

pub type Environment<T> = HashMap<Identifier, Type<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_validation() {
        assert!(Identifier::is_valid("f32_sdsa3_"));
        assert!(Identifier::is_valid("_fsd3_"));
        assert!(!Identifier::is_valid("4fd"));
        assert!(!Identifier::is_valid(""));
        assert!(!Identifier::is_valid("gdfg+gdf"));
    }

    #[test]
    fn test_identifier_new() {
        assert!(matches!(Identifier::new("pociąg"), Ok(_)));
        assert!(matches!(Identifier::new("32"), Err(_)));
    }
}
