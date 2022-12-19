#![allow(dead_code)]

use crate::traits::MatrixNumber;
use crate::traits::{CheckedMulScl, LaTeXable};
use anyhow::Context;
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, CheckedSub};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix<T: MatrixNumber> {
    data: Vec<Vec<T>>,
}

#[derive(Debug, Clone)]
struct Aftermath<T: MatrixNumber> {
    result: Matrix<T>,
    steps: Vec<String>,
}

impl<T: MatrixNumber> Matrix<T> {
    fn new(data: Vec<Vec<T>>) -> Self {
        Self { data }
    }

    pub fn new_safe(data: Vec<Vec<T>>) -> Self {
        let matrix = Self { data };
        matrix.get_shape().expect("Invalid matrix form");
        matrix
    }

    fn filled<F>((h, w): (usize, usize), supp: F) -> Self
    where
        F: Fn(usize, usize) -> T,
    {
        let mut res = Self::zeros((h, w)).data;
        for (i, item) in res.iter_mut().enumerate() {
            for (j, item_item) in item.iter_mut().enumerate() {
                let temp = std::mem::replace(item_item, T::zero());
                let _ = std::mem::replace(item_item, temp + supp(i, j));
            }
        }
        Self::new(res)
    }

    fn zeros((h, w): (usize, usize)) -> Self {
        let mut res = vec![];
        for _ in 0..h {
            res.push(vec![T::zero(); w]);
        }
        Self::new(res)
    }

    fn echelon(mut self) -> anyhow::Result<Aftermath<T>> {
        const CONTEXT: &str = "Calculations error!";

        if self.data.is_empty() {
            return Ok(Aftermath::<T> {
                result: self,
                steps: Vec::new(),
            });
        }

        let rows = self.data.len();
        let cols = self.data[0].len();

        let mut steps = vec![self.to_latex()];
        let mut c = 0;
        let mut i = 0;

        let mut add_step = |step: &str, matrix: &Matrix<T>| {
            steps.push(format!(r"\xrightarrow{{{}}} {}", step, matrix.to_latex()));
        };

        fn nice<T: MatrixNumber>(coef: &T) -> Option<i64> {
            if coef.is_zero() {
                Some(1000)
            } else if coef.is_one() {
                Some(0)
            } else if (T::zero().checked_sub(coef)?).is_one() {
                Some(1)
            } else {
                Some(2)
            }
        }

        fn sub_coeff_to_latex<T: MatrixNumber>(coef: &T) -> Option<String> {
            if coef.is_one() {
                Some("- ".to_string())
            } else if (T::zero().checked_sub(coef)?).is_one() {
                Some("+ ".to_string())
            } else if coef.is_positive() {
                Some(format!("- {}", coef.to_latex()))
            } else if coef.is_negative() {
                Some(format!("+ {}", (T::zero().checked_sub(coef)?.to_latex())))
            } else {
                unreachable!("Should not be used for zero coefficient!")
            }
        }

        while c < cols && i < rows {
            let mut j = i;
            for k in i + 1..rows {
                if nice(&self.data[k][c]).context(CONTEXT)?
                    < nice(&self.data[j][c]).context(CONTEXT)?
                {
                    j = k;
                }
            }

            if !self.data[j][c].is_zero() {
                if i != j {
                    self.data.swap(i, j);
                    add_step(
                        format!(r"w_{{{}}} \leftrightarrow w_{{{}}}", i + 1, j + 1).as_str(),
                        &self,
                    );
                }

                if !self.data[i][c].is_one() {
                    let d = self.data[i][c].clone();
                    for j in c..cols {
                        self.data[i][j] = self.data[i][j].checked_div(&d).context(CONTEXT)?;
                    }

                    add_step(
                        format!(r"w_{{{}}} : {}", i + 1, d.to_latex_single()).as_str(),
                        &self,
                    );
                }

                let mut step_ops: Vec<String> = Vec::new();
                for j in 0..rows {
                    if j != i && !self.data[j][c].is_zero() {
                        let p = self.data[j][c]
                            .checked_div(&self.data[i][c])
                            .context(CONTEXT)?;
                        for k in c..cols {
                            self.data[j][k] = self.data[j][k]
                                .checked_sub(&self.data[i][k].checked_mul(&p).context(CONTEXT)?)
                                .context(CONTEXT)?;
                        }

                        step_ops.push(format!(
                            "w_{{{}}} {}w_{{{}}}",
                            j + 1,
                            sub_coeff_to_latex(&p).context(CONTEXT)?,
                            i + 1
                        ));
                    }
                }

                if !step_ops.is_empty() {
                    add_step(
                        format!(r"\substack{{{}}}", &step_ops.join(r"\\")).as_str(),
                        &self,
                    );
                }

                i += 1;
            }

            c += 1;
        }

        Ok(Aftermath {
            result: self,
            steps,
        })
    }

    fn get_shape(&self) -> anyhow::Result<(usize, usize)> {
        let (mismatch, row_len) = self
            .data
            .iter()
            .skip(1)
            .fold((false, self.data[0].len()), |(acc, row_len), next| {
                (acc || row_len != next.len(), row_len)
            });

        if mismatch {
            anyhow::bail!("Invalid matrix! Bad shape!");
        }

        Ok((self.data.len(), row_len))
    }

    fn check_shape(&self, other: &Self) -> anyhow::Result<()> {
        let self_shape = self.get_shape()?;
        let other_shape = other.get_shape()?;
        if self_shape == other_shape {
            Ok(())
        } else {
            anyhow::bail!("Matrices have different shapes! {self_shape:?} != {other_shape:?}");
        }
    }

    fn check_shape_for_mul(&self, other: &Self) -> anyhow::Result<()> {
        let (_, self_w) = self.get_shape()?;
        let (other_h, _) = other.get_shape()?;
        if self_w == other_h {
            Ok(())
        } else {
            anyhow::bail!("Matrices have incompatible shapes! {self_w:?} != {other_h:?}");
        }
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
            .ok_or_else(|| anyhow::anyhow!("Operation failed!"))?;
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

    fn to_latex_single(&self) -> String {
        self.to_latex()
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

impl<T: MatrixNumber + Neg> Neg for Matrix<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.checked_operation(|a| Some(-a.clone()))
            .expect("Negation failed!")
    }
}

impl<T: MatrixNumber> Mul<Self> for Matrix<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self::CheckedMul::checked_mul(&self, &rhs).expect("Matrix multiplication failed!")
    }
}

impl<T: MatrixNumber> CheckedMul for Matrix<T> {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.check_shape_for_mul(v).ok()?;
        let (h, _) = self.get_shape().unwrap();
        let (_, w) = v.get_shape().unwrap();

        let mut res = Matrix::<T>::zeros((h, w)).data;
        for (i, item) in self.data.iter().enumerate() {
            for j in 0..w {
                for (k, item_item) in item.iter().enumerate() {
                    res[i][j] = res[i][j].clone() + item_item.clone() * v.data[k][j].clone();
                }
            }
        }
        Some(Self::new(res))
    }
}

impl<T: MatrixNumber> Mul<T> for Matrix<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        self.checked_mul_scl(&rhs).expect("Matrix multiplication failed!")
    }
}

impl<T: MatrixNumber> CheckedMulScl<T> for Matrix<T> {
    fn checked_mul_scl(&self, other: &T) -> Option<Self> {
        self.checked_operation(|a| a.checked_mul(other)).ok()
    }
}

#[macro_export]
macro_rules! rv {
    ($($x:expr),+ $(,)?) => (
        vec![
            $(ri!($x)),+
        ]
    );
}

#[macro_export]
macro_rules! rm {
    ($($($x:expr),+ $(,)?);+ $(;)?) => (
        Matrix::<Rational64>::new_safe(vec![
            $(rv!($($x),+)),+
        ])
    );
}

#[macro_export]
macro_rules! im {
    ($($($x:expr),+ $(,)?);+ $(;)?) => (
        Matrix::new_safe(vec![
            $(vec![
                $($x),+
            ]),+
        ])
    );
}

#[cfg(test)]
mod tests {
    use crate::ri;
    use num_rational::Rational64;

    use super::*;

    #[test]
    fn test_matrix() {
        let matrix = im![1, 2, 3; 4, 5, 6];
        assert_eq!(
            matrix.to_latex(),
            r"\begin{bmatrix}1 & 2 & 3\\4 & 5 & 6\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_addition() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2, 3; 4, 5, 6];

        let result = m + n;
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}2 & 4 & 6\\8 & 10 & 12\end{bmatrix}"
        );
    }

    #[test]
    fn test_echelon_rational1() {
        let m = rm![-2, 1; 1, 1];
        let expected = rm![1, 0; 0, 1];

        let aftermath = m.echelon().unwrap();

        assert_eq!(aftermath.result.to_latex(), expected.to_latex());
        assert_eq!(
            aftermath.steps,
            vec![
                r"\begin{bmatrix}-2 & 1\\1 & 1\end{bmatrix}",
                r"\xrightarrow{w_{1} \leftrightarrow w_{2}} \begin{bmatrix}1 & 1\\-2 & 1\end{bmatrix}",
                r"\xrightarrow{\substack{w_{2} + 2w_{1}}} \begin{bmatrix}1 & 1\\0 & 3\end{bmatrix}",
                r"\xrightarrow{w_{2} : 3} \begin{bmatrix}1 & 1\\0 & 1\end{bmatrix}",
                r"\xrightarrow{\substack{w_{1} - w_{2}}} \begin{bmatrix}1 & 0\\0 & 1\end{bmatrix}",
            ]
        );
    }

    #[test]
    fn test_echelon_rational2() {
        let m = rm![4, 3; 2, 1];
        let expected = rm![1, 0; 0, 1];

        let aftermath = m.echelon().unwrap();

        assert_eq!(aftermath.result.to_latex(), expected.to_latex());
        assert_eq!(
            aftermath.steps,
            vec![
                r"\begin{bmatrix}4 & 3\\2 & 1\end{bmatrix}",
                r"\xrightarrow{w_{1} : 4} \begin{bmatrix}1 & \frac{3}{4}\\2 & 1\end{bmatrix}",
                r"\xrightarrow{\substack{w_{2} - 2w_{1}}} \begin{bmatrix}1 & \frac{3}{4}\\0 & -\frac{1}{2}\end{bmatrix}",
                r"\xrightarrow{w_{2} : \left(-\frac{1}{2}\right)} \begin{bmatrix}1 & \frac{3}{4}\\0 & 1\end{bmatrix}",
                r"\xrightarrow{\substack{w_{1} - \frac{3}{4}w_{2}}} \begin{bmatrix}1 & 0\\0 & 1\end{bmatrix}",
            ]
        );
    }

    #[test]
    fn test_echelon_rational3() {
        let id = rm![1, 0; 0, 1];

        let aftermath = id.clone().echelon().unwrap();

        assert_eq!(aftermath.result.to_latex(), id.to_latex());
        assert_eq!(
            aftermath.steps,
            vec![r"\begin{bmatrix}1 & 0\\0 & 1\end{bmatrix}"]
        );
    }

    #[test]
    fn test_echelon_rational4() {
        let m = rm![1, -1, 1; 1, 1, -1; -1, 1, -1];
        let expected = rm![1, 0, 0; 0, 1, -1; 0, 0, 0];

        let aftermath = m.echelon().unwrap();

        assert_eq!(aftermath.result.to_latex(), expected.to_latex());
        assert_eq!(
            aftermath.steps,
            vec![
                r"\begin{bmatrix}1 & -1 & 1\\1 & 1 & -1\\-1 & 1 & -1\end{bmatrix}",
                r"\xrightarrow{\substack{w_{2} - w_{1}\\w_{3} + w_{1}}} \begin{bmatrix}1 & -1 & 1\\0 & 2 & -2\\0 & 0 & 0\end{bmatrix}",
                r"\xrightarrow{w_{2} : 2} \begin{bmatrix}1 & -1 & 1\\0 & 1 & -1\\0 & 0 & 0\end{bmatrix}",
                r"\xrightarrow{\substack{w_{1} + w_{2}}} \begin{bmatrix}1 & 0 & 0\\0 & 1 & -1\\0 & 0 & 0\end{bmatrix}",
            ]
        );
    }

    #[test]
    fn test_simple_subtraction() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2, 3; 4, 5, 6];

        let result = m - n;
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}0 & 0 & 0\\0 & 0 & 0\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_negation() {
        let m = im![1, 2, 3; 4, 5, 6];

        let result = -m;
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}-1 & -2 & -3\\-4 & -5 & -6\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_multiplication() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2; 3, 4; 5, 6];

        let result = m * n;
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}22 & 28\\49 & 64\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_multiplication_with_scalar() {
        let m = im![1, 2, 3; 4, 5, 6];

        let result = m * 2;
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}2 & 4 & 6\\8 & 10 & 12\end{bmatrix}"
        );
    }

    #[test]
    fn test_simple_multiplication_with_rational() {
        let m = rm![1, 2, 3; 4, 5, 6];

        let result = m * ri!(2);
        assert_eq!(
            result.to_latex(),
            r"\begin{bmatrix}2 & 4 & 6\\8 & 10 & 12\end{bmatrix}"
        );
    }
}
