use std::collections::btree_map::IterMut;
use std::collections::BTreeMap;

use anyhow::{bail, Context};

use crate::locale::Locale;
use crate::traits::{GuiDisplayable, LaTeXable};
use crate::{matrices::Matrix, traits::MatrixNumber};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Identifier {
    id: String,
}

impl Identifier {
    pub const RESULT: &'static str = "$";

    pub fn new(id: String) -> anyhow::Result<Self> {
        if Self::is_valid(&id) {
            Ok(Self { id })
        } else {
            bail!("Invalid identifier.")
        }
    }

    fn new_unsafe(id: String) -> Self {
        Self { id }
    }

    pub fn result() -> Self {
        Self {
            id: Self::RESULT.to_string(),
        }
    }

    pub fn is_result(&self) -> bool {
        self.id == Self::RESULT
    }

    pub fn is_valid(id: &str) -> bool {
        id.chars().all(|c| c.is_alphanumeric() || c == '_')
            && id.starts_with(|c: char| c.is_alphabetic() || c == '_')
    }
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<T: MatrixNumber> {
    Scalar(T),
    Matrix(Matrix<T>),
}

impl<T: MatrixNumber> Type<T> {
    pub fn from_scalar_option(opt: Option<T>) -> anyhow::Result<Self> {
        match opt {
            Some(val) => Ok(Self::Scalar(val)),
            None => Err(anyhow::anyhow!(
                "Arithmetic operation resulted in overflow!"
            )),
        }
    }

    pub fn from_matrix_result(opt: anyhow::Result<Matrix<T>>) -> anyhow::Result<Self> {
        Ok(Self::Matrix(opt?))
    }

    pub fn as_scalar(self) -> anyhow::Result<T> {
        match self {
            Type::Scalar(s) => Ok(s),
            Type::Matrix(_) => bail!("Expected scalar, got matrix."),
        }
    }

    pub fn as_matrix(self) -> anyhow::Result<Matrix<T>> {
        match self {
            Type::Matrix(m) => Ok(m),
            Type::Scalar(_) => bail!("Expected matrix, got scalar."),
        }
    }
}

impl<T: MatrixNumber> ToString for Type<T> {
    fn to_string(&self) -> String {
        match self {
            Type::Scalar(s) => s.to_string(),
            Type::Matrix(m) => m.to_string(),
        }
    }
}

impl<T: MatrixNumber> GuiDisplayable for Type<T> {
    fn display_string(&self, locale: &Locale) -> String {
        match self {
            Type::Scalar(s) => s.to_string(),
            Type::Matrix(m) => m.display_string(locale),
        }
    }

    fn to_shape(
        &self,
        ctx: &egui::Context,
        font_id: egui::FontId,
        color: egui::Color32,
    ) -> egui::Shape {
        match self {
            Type::Scalar(s) => s.to_shape(ctx, font_id, color),
            Type::Matrix(m) => m.to_shape(ctx, font_id, color),
        }
    }
}

impl<T: MatrixNumber> LaTeXable for Type<T> {
    fn to_latex(&self) -> String {
        match self {
            Type::Scalar(s) => s as &dyn LaTeXable,
            Type::Matrix(m) => m,
        }
        .to_latex()
    }
}

pub type Callable<T> = dyn Fn(Type<T>) -> anyhow::Result<Type<T>>;

fn builtin_functions<T: MatrixNumber>() -> BTreeMap<Identifier, Box<Callable<T>>> {
    BTreeMap::from([
        (
            Identifier::new_unsafe("transpose".to_string()),
            Box::new(|t: Type<T>| Ok(Type::Matrix(t.as_matrix()?.transpose()))) as Box<Callable<T>>,
        ),
        (
            Identifier::new_unsafe("identity".to_string()),
            Box::new(|t: Type<T>| {
                Ok(Type::Matrix(Matrix::identity(
                    t.as_scalar()?
                        .to_usize()
                        .context("Invalid identity argument")?,
                )))
            }) as Box<Callable<T>>,
        ),
        (
            Identifier::new_unsafe("inverse".to_string()),
            Box::new(|t: Type<T>| Ok(Type::Matrix(t.as_matrix()?.inverse()?.result)))
                as Box<Callable<T>>,
        ),
    ])
}

pub struct Environment<T: MatrixNumber> {
    env: BTreeMap<Identifier, Type<T>>,
    fun: BTreeMap<Identifier, Box<Callable<T>>>,
}

impl<T: MatrixNumber> Environment<T> {
    pub fn new() -> Self {
        Self {
            env: BTreeMap::new(),
            fun: builtin_functions(),
        }
    }

    pub fn insert(&mut self, id: Identifier, value: Type<T>) {
        self.env.insert(id, value);
    }

    pub fn get_value(&self, id: &Identifier) -> Option<&Type<T>> {
        self.env.get(id)
    }

    pub fn get_function(&self, id: &Identifier) -> Option<&Box<Callable<T>>> {
        self.fun.get(id)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Identifier, Type<T>> {
        self.env.iter_mut()
    }
}

impl<T: MatrixNumber> Default for Environment<T> {
    fn default() -> Self {
        Self::new()
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
        assert!(matches!(Identifier::new("".to_string()), Err(_)));
    }
}
