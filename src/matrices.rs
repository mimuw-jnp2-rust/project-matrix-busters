#![allow(dead_code)]

use crate::traits::MatrixNumber;
use crate::traits::{CheckedMulScl, LaTeXable};
use anyhow::{bail, Context};
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, CheckedSub};
use std::ops::{Add, Mul, Neg, Sub};

/// A matrix of type `T`.
/// Matrices are immutable.
/// Empty matrices have shape (0, 0), so be careful.
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
    /// Creates a new matrix from a 2D vector.
    /// The matrix is not checked for validity.
    /// If you want to check for validity, use `Matrix::new`.
    /// Calling methods on an invalid matrix will result in undefined behavior.
    /// # Arguments
    /// * `data` - The data of the matrix.
    /// # Returns
    /// A new matrix.
    /// # Examples
    /// ```
    /// let m = Matrix::new_unsafe(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    /// // m corresponds to the matrix
    /// // | 1 2 3 |
    /// // | 4 5 6 |
    /// ```
    pub fn new_unsafe(data: Vec<Vec<T>>) -> Self {
        Self { data }
    }

    /// Creates a new matrix from a 2D vector.
    /// The matrix is checked for validity.
    /// If you don't want to check for validity, use `Matrix::new_unsafe`.
    /// # Arguments
    /// * `data` - The data of the matrix.
    /// # Returns
    /// A new matrix.
    /// # Examples
    /// ```
    /// let m = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    /// // m corresponds to the matrix
    /// // | 1 2 3 |
    /// // | 4 5 6 |
    /// ```
    /// # Errors
    /// If the data is not a valid matrix.
    /// ```
    /// let m = Matrix::new(vec![vec![1, 2, 3], vec![4, 5]]);
    /// // m is an error
    /// ```
    pub fn new(data: Vec<Vec<T>>) -> anyhow::Result<Self> {
        let matrix = Self { data };
        if !matrix.is_valid() {
            bail!("Invalid matrix.")
        } else if matrix.is_empty() {
            Ok(Self::empty())
        } else {
            Ok(matrix)
        }
    }

    /// Creates a new matrix of shape (h, w) with given supplier.
    /// # Arguments
    /// * `(h, w)` - The shape of the matrix - height and width.
    /// * `supplier` - The supplier of the matrix.
    /// # Returns
    /// A new matrix of shape (h, w) with given supplier.
    /// # Examples
    /// ```
    /// let m = Matrix::new_with(2, 3, |i, j| i + j);
    /// // m corresponds to the matrix
    /// // | 0 1 2 |
    /// // | 1 2 3 |
    /// let m = Matrix::new_with(2, 3, |i, j| i * j);
    /// // m corresponds to the matrix
    /// // | 0 0 0 |
    /// // | 0 1 2 |
    /// ```
    pub fn filled<F>((h, w): (usize, usize), supp: F) -> Self
    where
        F: Fn(usize, usize) -> T,
    {
        if h == 0 || w == 0 {
            return Self::empty();
        }
        let mut data = vec![vec![T::zero(); w]; h];
        for (i, row) in data.iter_mut().enumerate().take(h) {
            for (j, elem) in row.iter_mut().enumerate().take(w) {
                *elem = supp(i, j);
            }
        }
        Self { data }
    }

    /// Creates zero matrix of shape (h, w).
    /// # Arguments
    /// * `(h, w)` - The shape of the matrix - height and width.
    /// # Returns
    /// A zero matrix of shape (h, w).
    /// # Examples
    /// ```
    /// let m = Matrix::zero(2, 3);
    /// // m corresponds to the matrix
    /// // | 0 0 0 |
    /// // | 0 0 0 |
    /// ```
    pub fn zeros((h, w): (usize, usize)) -> Self {
        Self::filled((h, w), |_, _| T::zero())
    }

    /// Creates identity (square) matrix of shape (n, n).
    /// # Arguments
    /// * `n` - The length of the side of the matrix.
    /// # Returns
    /// An identity matrix of shape (n, n).
    /// # Examples
    /// ```
    /// let m = Matrix::identity(3);
    /// // m corresponds to the matrix
    /// // | 1 0 0 |
    /// // | 0 1 0 |
    /// // | 0 0 1 |
    /// ```
    fn identity(n: usize) -> Self {
        Self::filled((n, n), |i, j| if i == j { T::one() } else { T::zero() })
    }

    /// Creates empty matrix of shape (0, 0).
    /// # Returns
    /// An empty matrix of shape (0, 0).
    fn empty() -> Self {
        Self::new_unsafe(vec![])
    }

    /// TODO: doc
    /// TODO: maybe move to matrix_algorithm.rs
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

    /// Returns the shape of the matrix.
    /// If the matrix is not valid, the behavior is undefined.
    /// If the matrix is empty, the shape is (0, 0).
    /// # Returns
    /// A tuple of the form `(height, width)`.
    /// # Examples
    /// ```
    /// let m = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// assert_eq!(m.get_shape(), (2, 3));
    /// ```
    fn get_shape(&self) -> (usize, usize) {
        if self.data.is_empty() {
            (0, 0)
        } else {
            (self.data.len(), self.data[0].len())
        }
    }

    /// Checks if matrix is empty.
    /// Matrix is empty if it has no rows or no columns.
    /// Matrix has to be valid. Otherwise, the behavior is undefined.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert!(!m.is_empty());
    /// let m = Matrix::new(vec![vec![], vec![], vec![]]).unwrap();
    /// assert!(m.is_empty());
    /// let m = Matrix::new(vec![]).unwrap();
    /// assert!(m.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.data.is_empty() || self.data[0].is_empty()
    }

    /// Checks if matrix is valid.
    /// Matrix is valid if all rows have the same length.
    /// # Examples
    /// ```rust
    /// let m = Matrix::new_unsafe(vec![vec![1, 2], vec![3, 4]]);
    /// assert!(m.is_valid());
    /// let m = Matrix::new_unsafe(vec![vec![1, 2], vec![3, 4, 5]]);
    /// assert!(!m.is_valid());
    /// ```
    fn is_valid(&self) -> bool {
        return if self.data.is_empty() {
            true
        } else {
            !self
                .data
                .iter()
                .skip(1)
                .fold((false, self.data[0].len()), |(acc, row_len), next| {
                    (acc || row_len != next.len(), row_len)
                })
                .0 // does any row have different length?
        };
    }

    /// Checks if two matrices have the same shape.
    /// Shape of a matrix is a tuple of the form `(height, width)`.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// `true` if the matrices have the same shape, `false` otherwise.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert!(m1.has_same_shape(&m2));
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// assert!(!m1.has_same_shape(&m2));
    /// ```
    fn same_shapes(&self, other: &Self) -> bool {
        let self_shape = self.get_shape();
        let other_shape = other.get_shape();
        self_shape == other_shape
    }

    /// Checks if two matrices can be multiplied.
    /// Matrices can be multiplied if the number of columns of the first matrix
    /// is equal to the number of rows of the second matrix.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// `true` if the matrices can be multiplied, `false` otherwise.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// assert!(m1.can_multiply(&m2));
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// assert!(!m1.can_multiply(&m2));
    /// ```
    fn corresponding_shapes_for_mul(&self, other: &Self) -> bool {
        let (_, self_w) = self.get_shape();
        let (other_h, _) = other.get_shape();
        self_w == other_h
    }

    /// Performs element-wise operation on two matrices.
    /// # Arguments
    /// * `other` - The other matrix.
    /// * `op` - The operation to perform.
    /// # Returns
    /// A new matrix with the result of the operation.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m3 = m1.checked_operation_on_two(&m2, |a, b| a + b).unwrap();
    /// assert_eq!(m3, Matrix::new(vec![vec![2, 4], vec![6, 8]]).unwrap());
    /// ```
    fn checked_operation_on_two<F>(&self, other: &Self, operation: F) -> anyhow::Result<Self>
    where
        F: Fn(&T, &T) -> Option<T>,
    {
        self.same_shapes(other)
            .then_some(())
            .context("Matrices have different shapes!")?;
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
        Self::new(data)
    }

    /// Performs matrix operation element-wise.
    /// # Arguments
    /// * `op` - The operation to perform.
    /// # Returns
    /// A new matrix with the result of the operation.
    /// # Examples
    /// ```rust
    /// let m = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = m.checked_operation(|a| a + 1).unwrap();
    /// assert_eq!(m2, Matrix::new(vec![vec![2, 3], vec![4, 5]]).unwrap());
    /// ```
    fn checked_operation<F>(&self, operation: F) -> anyhow::Result<Self>
    where
        F: Fn(&T) -> Option<T>,
    {
        // In order to avoid code duplication, we use `checked_operation_on_two`
        //  with `self` as the second argument and ignore it later.
        //  It is more memory efficient as we don't need to allocate a new matrix.
        //  First approach was to use some mock matrix, but it was less efficient.
        self.checked_operation_on_two(self, |a, _| operation(a))
    }

    /// Performs matrix to the power.
    /// # Arguments
    /// * `exponent` - The power to raise the matrix to.
    /// # Returns
    /// $M^{exponent}$ where $M$ is the matrix.
    pub fn checked_pow(&self, mut exponent: usize) -> anyhow::Result<Self> {
        let (h, w) = self.get_shape();
        if h != w {
            bail!("Only square matrices can be used in exponentiation!");
        }

        let mut pow2 = self.clone();
        let mut result = Self::identity(h);
        while exponent > 0 {
            if exponent % 2 == 1 {
                result = result
                    .checked_mul(&pow2)
                    .context("Multiplication failed!")?;
            }
            pow2 = pow2.checked_mul(&pow2).context("Multiplication failed!")?;
            exponent /= 2;
        }

        Ok(result)
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
        self.corresponding_shapes_for_mul(v).then_some(())?;
        let (h, _) = self.get_shape();
        let (_, w) = v.get_shape();

        let mut res = Matrix::<T>::zeros((h, w)).data;
        for (i, item) in self.data.iter().enumerate() {
            for j in 0..w {
                for (k, item_item) in item.iter().enumerate() {
                    res[i][j] = res[i][j].clone() + item_item.clone() * v.data[k][j].clone();
                }
            }
        }
        Self::new(res).ok()
    }
}

impl<T: MatrixNumber> Mul<T> for Matrix<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        self.checked_mul_scl(&rhs)
            .expect("Matrix multiplication failed!")
    }
}

impl<T: MatrixNumber> CheckedMulScl<T> for Matrix<T> {
    fn checked_mul_scl(&self, other: &T) -> Option<Self> {
        self.checked_operation(|a| a.checked_mul(other)).ok()
    }
}

/// Create a matrix row (vector) of Rational64 numbers passed as integers.
/// Uses ri! macro.
/// Used as helper macro for rm! macro.
/// rv stands for Rational Vector.
/// Example:
/// ```
/// rv!(1, 2, 3); // Creates a row vector [ri!(1), ri!(2), ri!(3)]
/// ```
#[macro_export]
macro_rules! rv {
    ($($x:expr),+ $(,)?) => (
        vec![
            $(ri!($x)),+
        ]
    );
}

/// Create a matrix of Rational64 numbers passed as integers.
/// Uses ri! and rv! macros.
/// rm stands for Rational Matrix.
/// Example:
/// ```
/// // Creates a matrix
/// // | 1 2 3 |
/// // | 4 5 6 |
/// // values of the matrix are Rational64 numbers
/// m!(1, 2, 3; 4, 5, 6);
/// ```
#[macro_export]
macro_rules! rm {
    ($($($x:expr),+ $(,)?);+ $(;)?) => (
        Matrix::<Rational64>::new_unsafe(vec![
            $(rv!($($x),+)),+
        ])
    );
}

/// Create a matrix row (vector) of i32 numbers passed as integers.
/// im stands for Integer Matrix.
/// Example:
/// ```
/// // Creates a matrix
/// // | 1 2 3 |
/// // | 4 5 6 |
/// // values of the matrix are i32 numbers
/// im!(1, 2, 3; 4, 5, 6);
/// ```
#[macro_export]
macro_rules! im {
    ($($($x:expr),+ $(,)?);+ $(;)?) => (
        Matrix::new_unsafe(vec![
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

    #[test]
    fn test_simple_exponentiation() {
        let m = im![1, 1; 1, 0];
        assert_eq!(m.checked_pow(0).unwrap(), im![1, 0; 0, 1]);
        assert_eq!(m.checked_pow(1).unwrap(), im![1, 1; 1, 0]);
        assert_eq!(m.checked_pow(2).unwrap(), im![2, 1; 1, 1]);
        assert_eq!(m.checked_pow(9).unwrap(), im![55, 34; 34, 21]);
        assert_eq!(m.checked_pow(10).unwrap(), im![89, 55; 55, 34]);

        let m2 = Matrix::new(vec![
            vec![Rational64::new(1, 1), Rational64::new(1, 2)],
            vec![Rational64::new(1, 3), Rational64::new(1, 4)],
        ])
        .context("Failed to create matrix - something is wrong with the test")
        .unwrap();
        assert_eq!(m2.checked_pow(0).unwrap(), rm![1, 0; 0, 1]);
        assert_eq!(m2.checked_pow(1).unwrap(), m2);
        assert_eq!(m2.checked_pow(2).unwrap(), m2.clone() * m2.clone());
        assert_eq!(
            m2.checked_pow(3).unwrap(),
            m2.clone() * m2.clone() * m2.clone()
        );
    }
}
