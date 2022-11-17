#![allow(dead_code)]

use crate::traits::LaTeXable;
use crate::traits::MatrixNumber;
use num_traits::{CheckedAdd, CheckedNeg, CheckedSub};
use std::ops::{Add, Neg, Sub};

struct Matrix<T: MatrixNumber> {
    data: Vec<Vec<T>>,
}

struct Aftermath<T: MatrixNumber> {
    result: Matrix<T>,
    steps: Vec<String>,
}

impl<T: MatrixNumber> Matrix<T> {
    fn new(data: Vec<Vec<T>>) -> Self {
        Self { data }
    }

    fn echelon(self) -> anyhow::Result<Aftermath<T>> {
        todo!("echelonisation not implemented!")
    }

    fn check_shape(&self, other: &Self) -> anyhow::Result<()> {
        if self.data.len() != other.data.len() {
            return Err(anyhow::anyhow!("Matrices have different number of rows!"));
        }

        if self.data.len() == 0 {
            return Ok(());
        }

        let (mismatch, _) =
            self.data
                .iter()
                .skip(1)
                .fold((true, &self.data[0]), |(acc, row), next| {
                    if acc && row.len() == next.len() {
                        (true, next)
                    } else {
                        (false, next)
                    }
                });

        if mismatch {
            return Err(anyhow::anyhow!(
                "Matrices have different number of columns!"
            ));
        }

        let shape_mismatch = self
            .data
            .iter()
            .zip(other.data.iter())
            .any(|(row1, row2)| row1.len() != row2.len());

        if shape_mismatch {
            return Err(anyhow::anyhow!("Matrices have different shapes!"));
        }

        Ok(())
    }

    fn checked_operation_on_two<F>(&self, other: &Self, operation: F) -> anyhow::Result<Self>
    where
        F: Fn(&T, &T) -> Option<T>,
    {
        self.check_shape(other)?;
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(row_self, row_other)| {
                row_self
                    .iter()
                    .zip(row_other.iter())
                    .map(|(elem_self, elem_other)| operation(elem_self, elem_other))
                    .collect::<Option<Vec<T>>>()
            })
            .collect::<Option<Vec<Vec<T>>>>()
            .ok_or(anyhow::anyhow!("Operation failed!"))?;
        Ok(Self::new(data))
    }

    fn checked_operation<F>(&self, operation: F) -> anyhow::Result<Self>
    where
        F: Fn(&T) -> Option<T>,
    {
        // We do a little trick and apply `self` twice, as it is more memory efficient
        self.checked_operation_on_two(self, |a, _| operation(a))
    }
}

// TODO: impl CheckedAdd, CheckedSub, CheckedMul

impl<T: MatrixNumber> LaTeXable for Matrix<T> {
    fn to_latex(&self) -> String {
        r"\begin{bmatrix}".to_string()
            + &self
                .data
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|elem| elem.to_latex())
                        .collect::<Vec<_>>()
                        .join(" & ")
                })
                .collect::<Vec<_>>()
                .join(r"\\")
            + r"\end{bmatrix}"
    }
}

impl<T: MatrixNumber> Add for Matrix<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(&rhs).expect("Addition failed!")
    }
}

impl<T: MatrixNumber> CheckedAdd for Matrix<T> {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.checked_operation_on_two(rhs, |a, b| a.checked_add(b))
            .ok()
    }
}

impl<T: MatrixNumber> Sub for Matrix<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(&rhs).expect("Matrix subtraction failed!")
    }
}

impl<T: MatrixNumber> CheckedSub for Matrix<T> {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.checked_operation_on_two(v, |a, b| a.checked_sub(b))
            .ok()
    }
}

impl<T: MatrixNumber + CheckedNeg> CheckedNeg for Matrix<T> {
    fn checked_neg(&self) -> Option<Self> {
        self.checked_operation(|a| a.checked_neg()).ok()
    }
}

impl<T: MatrixNumber + Neg> Neg for Matrix<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.checked_operation(|a| Some(-a.clone())).expect("Negation failed!")
    }
}

impl LaTeXable for i32 {
    fn to_latex(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix() {
        let matrix = Matrix::<i32>::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(
            matrix.to_latex(),
            r"\begin{bmatrix}1 & 2 & 3\\4 & 5 & 6\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_addition() {
        let m = Matrix::<i32>::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let n = Matrix::<i32>::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }
}
