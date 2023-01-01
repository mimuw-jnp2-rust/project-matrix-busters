#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::bail;

use crate::{matrices::Matrix, traits::MatrixNumber};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    id: String,
}

impl Identifier {
    pub fn new(id: String) -> anyhow::Result<Self> {
        if Self::is_valid(&id) {
            Ok(Self { id })
        } else {
            bail!("Invalid identifier.")
        }
    }

    fn is_valid(id: &str) -> bool {
        id.chars().all(|c| c.is_alphanumeric() || c == '_')
            && id.starts_with(|c: char| c.is_alphabetic() || c == '_')
    }

    pub fn get(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<T: MatrixNumber> {
    Scalar(T),
    Matrix(Matrix<T>),
}

pub struct Environment<T: MatrixNumber> {
    env: HashMap<Identifier, Type<T>>,
}

impl<T: MatrixNumber> Environment<T> {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: Identifier, value: Type<T>) {
        self.env.insert(id, value);
    }

    pub fn get(&self, id: &Identifier) -> Option<&Type<T>> {
        self.env.get(id)
    }
}

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
        assert!(matches!(Identifier::new("pociąg".to_string()), Ok(_)));
        assert!(matches!(Identifier::new("32".to_string()), Err(_)));
    }
}
