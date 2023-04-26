use crate::constants::{MATRIX_HPADDING, MATRIX_VPADDING};
use crate::traits::{BoxedShape, LaTeXable};
use crate::traits::{GuiDisplayable, MatrixNumber};
use anyhow::{bail, Context};
use egui::{pos2, Color32, FontId, Rect};
use locale::Locale;
use std::ops::{Add, Mul, Neg, Sub};

/// A matrix of type `T`.
/// Matrices are immutable.
/// Empty matrices have shape (0, 0), so be careful.
#[derive(Debug, Clone, Default)]
pub struct Matrix<T: MatrixNumber> {
    data: Vec<Vec<T>>,

    /// Index of a column that is followed by a vertical separator (counting
    /// from 0). This only affects exporting to LaTeX.
    separator: Option<usize>,
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
        Self {
            data,
            separator: None,
        }
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
        let matrix = Self {
            data,
            separator: None,
        };
        if !matrix.is_valid() {
            bail!("Invalid matrix.")
        } else if matrix.is_empty() {
            Ok(Self::empty())
        } else {
            Ok(matrix)
        }
    }

    /// Creates a new matrix from a 2D vector.
    /// The matrix is checked for validity.
    /// The initial data is passed as a vector of flattened rows.
    /// The shape of the matrix is passed as a tuple.
    /// # Arguments
    /// * `data` - The data of the matrix.
    /// * `(rows, cols)` - The shape of the matrix.
    /// # Returns
    /// A new matrix.
    /// # Examples
    /// ```
    /// let m = Matrix::from_vec(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// // m corresponds to the matrix
    /// // | 1 2 3 |
    /// // | 4 5 6 |
    /// ```
    /// # Errors
    /// If the data is not a valid matrix.
    pub fn from_vec(data: Vec<T>, (rows, cols): (usize, usize)) -> anyhow::Result<Self> {
        if data.len() != rows * cols {
            bail!("Invalid size.")
        } else if rows == 0 || cols == 0 {
            Ok(Self::empty())
        } else {
            Self::new(data.chunks(cols).map(|c| c.to_vec()).collect())
        }
    }

    /// Gets the index of a column that is followed by a vertical separator (if any).
    /// # Returns
    /// The index of the separator (or None if there is no separator).
    pub fn get_separator(&self) -> Option<usize> {
        self.separator
    }

    /// Sets the index of a column that is followed by a vertical separator
    /// (counting from 0).
    /// # Arguments
    /// * `separator` - The index of the separator (or None if there is no separator).
    /// # Examples
    /// ```
    /// let mut m = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    /// // m corresponds to the matrix
    /// // | 1 2 3 |
    /// // | 4 5 6 |
    /// m.set_separator(Some(1));
    /// // m corresponds to the matrix
    /// // | 1 2 | 3 |
    /// // | 4 5 | 6 |
    /// ```
    pub fn set_separator(&mut self, separator: Option<usize>) {
        self.separator = separator;
    }

    /// Creates a new matrix by setting the index of a column that is followed
    /// by a vertical separator (counting from 0).
    /// # Arguments
    /// * `separator` - The index of the separator (or None if there is no separator).
    /// # Returns
    /// A new matrix.
    pub fn with_separator(mut self, separator: Option<usize>) -> Self {
        self.set_separator(separator);
        self
    }

    /// Creates a new matrix by reshaping an existing matrix.
    /// If new shape is not compatible with the old shape, an error is returned.
    /// # Arguments
    /// * `matrix` - The matrix to reshape.
    /// * `(rows, cols)` - The new shape of the matrix.
    /// # Returns
    /// A new matrix.
    /// # Examples
    /// ```
    /// let m = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    /// // m corresponds to the matrix
    /// // | 1 2 3 |
    /// // | 4 5 6 |
    /// let m = Matrix::reshape(m, (3, 2));
    /// // m corresponds to the matrix
    /// // | 1 2 |
    /// // | 3 4 |
    /// // | 5 6 |
    /// ```
    pub fn reshape(&self, (rows, cols): (usize, usize)) -> anyhow::Result<Self> {
        let (h, w) = self.get_shape();
        if h * w != rows * cols {
            bail!("Invalid size.")
        } else {
            Self::from_vec(self.data.iter().flatten().cloned().collect(), (rows, cols))
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
        Self {
            data,
            separator: None,
        }
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

    /// Creates ones matrix of shape (h, w).
    /// # Arguments
    /// * `(h, w)` - The shape of the matrix - height and width.
    /// # Returns
    /// A ones matrix of shape (h, w).
    /// # Examples
    /// ```
    /// let m = Matrix::ones(2, 3);
    /// // m corresponds to the matrix
    /// // | 1 1 1 |
    /// // | 1 1 1 |
    /// ```
    pub fn ones((h, w): (usize, usize)) -> Self {
        Self::filled((h, w), |_, _| T::one())
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
    pub fn identity(n: usize) -> Self {
        Self::filled((n, n), |i, j| if i == j { T::one() } else { T::zero() })
    }

    /// Creates empty matrix of shape (0, 0).
    /// # Returns
    /// An empty matrix of shape (0, 0).
    pub fn empty() -> Self {
        Self::new_unsafe(vec![])
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
    pub fn get_shape(&self) -> (usize, usize) {
        if self.is_empty() {
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
    pub fn is_empty(&self) -> bool {
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
    pub fn is_valid(&self) -> bool {
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

    /// Returns the raw data of the matrix.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert_eq!(m.get_data(), &vec![vec![1, 2], vec![3, 4]]);
    /// ```
    pub fn get_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    /// Returns the raw data of the matrix and consumes the matrix.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert_eq!(m.into_data(), vec![vec![1, 2], vec![3, 4]]);
    /// ```
    pub fn consume(self) -> Vec<Vec<T>> {
        self.data
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
    pub fn same_shapes(&self, other: &Self) -> bool {
        let self_shape = self.get_shape();
        let other_shape = other.get_shape();
        self_shape == other_shape
    }

    /// Return the shape of a matrix after multiplication.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// A tuple of the form `(height, width)`.
    /// # Errors
    /// Returns `Err` if the matrices cannot be multiplied - e.g. if they have incompatible shapes.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    /// assert_eq!(m1.get_shape_after_mul(&m2), Ok((2, 2)));
    /// let m1 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert!(m1.get_shape_after_mul(&m2).is_err());
    /// ```
    fn result_shape_for_mul(&self, other: &Self) -> anyhow::Result<(usize, usize)> {
        let (h, self_w) = self.get_shape();
        let (other_h, w) = other.get_shape();
        if self_w == other_h {
            Ok((h, w))
        } else {
            Err(anyhow::anyhow!(
                "Cannot multiply matrices of shapes ({h}, {self_w}) and ({other_h}, {w})"
            ))
        }
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
    pub fn checked_operation_on_two<F>(&self, other: &Self, operation: F) -> anyhow::Result<Self>
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
    pub fn checked_operation<F>(&self, operation: F) -> anyhow::Result<Self>
    where
        F: Fn(&T) -> Option<T>,
    {
        // In order to avoid code duplication, we use `checked_operation_on_two`
        //  with `self` as the second argument and ignore it later.
        //  It is more memory efficient as we don't need to allocate a new matrix.
        //  First approach was to use some mock matrix, but it was less efficient.
        self.checked_operation_on_two(self, |a, _| operation(a))
    }

    /// Performs matrix addition.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// A new matrix with the result of the addition.
    /// # Errors
    /// Returns `Err` if the matrices have different shapes.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m3 = m1.checked_add(&m2).unwrap();
    /// assert_eq!(m3, Matrix::new(vec![vec![2, 4], vec![6, 8]]).unwrap());
    /// ```
    pub fn checked_add(&self, rhs: &Self) -> anyhow::Result<Self> {
        self.checked_operation_on_two(rhs, |a, b| a.checked_add(b))
    }

    /// Performs matrix subtraction.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// A new matrix with the result of the subtraction.
    /// # Errors
    /// Returns `Err` if the matrices have different shapes.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m3 = m1.checked_sub(&m2).unwrap();
    /// assert_eq!(m3, Matrix::new(vec![vec![0, 0], vec![0, 0]]).unwrap());
    /// ```
    pub fn checked_sub(&self, v: &Self) -> anyhow::Result<Self> {
        self.checked_operation_on_two(v, |a, b| a.checked_sub(b))
    }

    /// Performs matrix negation.
    /// # Returns
    /// A new matrix with the result of the negation.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = m1.checked_neg().unwrap();
    /// assert_eq!(m2, Matrix::new(vec![vec![-1, -2], vec![-3, -4]]).unwrap());
    /// ```
    pub fn checked_neg(&self) -> anyhow::Result<Self> {
        Self::zeros(self.get_shape()).checked_sub(self)
    }

    /// Performs matrix multiplication.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// A new matrix with the result of the multiplication.
    /// # Errors
    /// Returns `Err` if height of the first matrix is not equal to the width of the second matrix.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m3 = m1.checked_mul(&m2).unwrap();
    /// assert_eq!(m3, Matrix::new(vec![vec![7, 10], vec![15, 22]]).unwrap());
    /// ```
    pub fn checked_mul(&self, v: &Self) -> anyhow::Result<Self> {
        const OVERFLOW_MSG: &str = "Overflow during matrix multiplication!";
        let (h, w) = self.result_shape_for_mul(v)?;

        let mut res = Matrix::<T>::zeros((h, w)).data;
        for (i, item) in self.data.iter().enumerate() {
            for j in 0..w {
                for (k, item_item) in item.iter().enumerate() {
                    res[i][j] = (item_item.checked_mul(&v.data[k][j]).context(OVERFLOW_MSG)?)
                        .checked_add(&res[i][j])
                        .context(OVERFLOW_MSG)?;
                }
            }
        }
        Self::new(res)
    }

    /// Performs matrix multiplication by a scalar.
    /// # Arguments
    /// * `other` - The scalar.
    /// # Returns
    /// A new matrix with the result of the multiplication.
    /// # Errors
    /// Returns `Err` if the multiplication overflows.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = m1.checked_mul_scl(&2).unwrap();
    /// assert_eq!(m2, Matrix::new(vec![vec![2, 4], vec![6, 8]]).unwrap());
    /// ```
    pub fn checked_mul_scl(&self, other: &T) -> anyhow::Result<Self> {
        self.checked_operation(|a| a.checked_mul(other))
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

    /// Concats two matrices horizontally. Assumes that both matrices have the
    /// same number of rows.
    /// # Arguments
    /// * `other` - The other matrix.
    /// # Returns
    /// A new matrix with the result of the operation.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// let m2 = Matrix::new(vec![vec![5, 6], vec![7, 8]]).unwrap();
    /// let m3 = m1.concat(m2).unwrap();
    /// assert_eq!(m3, Matrix::new(vec![vec![1, 2, 5, 6], vec![3, 4, 7, 8]]).unwrap());
    /// ```
    pub fn concat(mut self, other: Self) -> anyhow::Result<Self> {
        let (rows, columns) = self.get_shape();
        if rows != other.get_shape().0 {
            bail!("Cannot concatenate matrices with different number of rows!");
        }

        std::iter::zip(self.data.iter_mut(), other.data.into_iter())
            .for_each(|(a, b)| a.extend(b.into_iter()));
        Ok(self.with_separator(Some(columns)))
    }

    /// Splits the matrix horizontally at the given column. Drops the separator.
    /// # Arguments
    /// * `column` - The column to split at.
    /// # Returns
    /// A tuple of two matrices.
    /// # Examples
    /// ```rust
    /// use matrix::Matrix;
    /// let m1 = Matrix::new(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]).unwrap();
    /// let (m2, m3) = m1.split(2).unwrap();
    /// assert_eq!(m2, Matrix::new(vec![vec![1, 2], vec![5, 6]]).unwrap());
    /// assert_eq!(m3, Matrix::new(vec![vec![3, 4], vec![7, 8]]).unwrap());
    /// ```
    pub fn split(mut self, column: usize) -> anyhow::Result<(Self, Self)> {
        let (_, columns) = self.get_shape();
        if column > columns {
            bail!("Cannot split matrix at column {}!", column);
        }

        let right = Self::new_unsafe(
            self.data
                .iter_mut()
                .map(|row| row.split_off(column))
                .collect(),
        );
        self.separator = None;
        Ok((self, right))
    }
}

impl<T: MatrixNumber> PartialEq for Matrix<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_empty() && other.is_empty() {
            return true;
        }
        if self.get_shape() != other.get_shape() {
            return false;
        }
        self.data == other.data
    }
}

impl<T: MatrixNumber> Eq for Matrix<T> {}

impl<T: MatrixNumber> LaTeXable for Matrix<T> {
    fn to_latex(&self) -> String {
        let mut column_format = "c".repeat(self.data[0].len());
        if let Some(s) = self.separator {
            column_format.insert(s, '|')
        }

        format!(
            r"\left[\begin{{array}}{{{}}}{}\end{{array}}\right]",
            column_format,
            &self
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
        )
    }

    fn to_latex_single(&self) -> String {
        self.to_latex()
    }
}

impl<T: MatrixNumber> GuiDisplayable for Matrix<T> {
    fn display_string(&self, locale: &Locale) -> String {
        let (h, w) = self.get_shape();
        let name = locale.get_translated("matrix");
        format!("{name}::<{h}, {w}>")
    }

    fn to_shape(&self, ctx: &egui::Context, font_id: FontId, color: Color32) -> egui::Shape {
        let (rows, cols) = self.get_shape();
        let mut shapes: Vec<Vec<egui::Shape>> = self
            .get_data()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|element| element.to_shape(ctx, font_id.clone(), color))
                    .collect()
            })
            .collect();

        let mut row_heights = vec![0_f32; rows];
        let mut column_widths = vec![0_f32; cols];
        for (i, row) in shapes.iter().enumerate() {
            for (j, shape) in row.iter().enumerate() {
                let rect = shape.get_rect();
                row_heights[i] = row_heights[i].max(rect.height());
                column_widths[j] = column_widths[j].max(rect.width());
            }
        }

        let mut upper_left = pos2(0., 0.);
        for (i, row) in shapes.iter_mut().enumerate() {
            for (j, shape) in row.iter_mut().enumerate() {
                let rect = shape.get_rect().size();
                shape.translate(
                    egui::Align2::CENTER_CENTER
                        .align_size_within_rect(
                            rect,
                            Rect {
                                min: upper_left,
                                max: pos2(
                                    upper_left.x + column_widths[j],
                                    upper_left.y + row_heights[i],
                                ),
                            },
                        )
                        .min
                        .to_vec2(),
                );
                upper_left.x += column_widths[j] + MATRIX_HPADDING;
            }
            upper_left.x = 0.;
            upper_left.y += row_heights[i] + MATRIX_VPADDING;
        }

        egui::Shape::Vec(shapes.into_iter().flat_map(|row| row.into_iter()).collect())
    }
}

impl<T: MatrixNumber> Add for Matrix<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(&rhs).expect("Addition failed!")
    }
}

impl<T: MatrixNumber> Sub for Matrix<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(&rhs).expect("Matrix subtraction failed!")
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
        self.checked_mul(&rhs)
            .expect("Matrix multiplication failed!")
    }
}

impl<T: MatrixNumber> Mul<T> for Matrix<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        self.checked_mul_scl(&rhs)
            .expect("Matrix multiplication failed!")
    }
}

impl<T: MatrixNumber> ToString for Matrix<T> {
    fn to_string(&self) -> String {
        self.data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<T: MatrixNumber> From<Matrix<T>> for Vec<T> {
    fn from(value: Matrix<T>) -> Self {
        value.data.into_iter().flatten().collect()
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
/// rm!(1, 2, 3; 4, 5, 6);
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
            r"\left[\begin{array}{ccc}1 & 2 & 3\\4 & 5 & 6\end{array}\right]"
        );
    }

    #[test]
    fn test_new_unsafe() {
        let matrix = Matrix::new_unsafe(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let matrix2 = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
        assert_eq!(matrix, matrix2);
    }

    #[test]
    fn test_new() {
        let matrix = Matrix::new(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
        assert_eq!(matrix, im![1, 2, 3; 4, 5, 6]);
        let invalid = Matrix::new(vec![vec![1, 2, 3], vec![4, 5]]);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_is_valid() {
        let matrix = Matrix::new_unsafe(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        assert!(matrix.is_valid());
        let matrix = Matrix::new_unsafe(vec![vec![1, 2, 3], vec![4, 5]]);
        assert!(!matrix.is_valid());
    }

    #[test]
    fn test_filled() {
        let matrix = Matrix::filled((4, 5), |x, y| ri!((x * y) as i64));
        assert_eq!(
            matrix,
            rm![
                0, 0, 0, 0, 0;
                0, 1, 2, 3, 4;
                0, 2, 4, 6, 8;
                0, 3, 6, 9, 12;
            ]
        );
        let matrix = Matrix::filled((4, 5), |x, y| ri!((x + y) as i64));
        assert_eq!(
            matrix,
            rm![
                0, 1, 2, 3, 4;
                1, 2, 3, 4, 5;
                2, 3, 4, 5, 6;
                3, 4, 5, 6, 7;
            ]
        );
    }

    #[test]
    fn test_zeros() {
        let matrix = Matrix::zeros((4, 5));
        assert_eq!(
            matrix,
            rm![
                0, 0, 0, 0, 0;
                0, 0, 0, 0, 0;
                0, 0, 0, 0, 0;
                0, 0, 0, 0, 0;
            ]
        );
    }

    #[test]
    fn test_ones() {
        let matrix = Matrix::ones((4, 5));
        assert_eq!(
            matrix,
            rm![
                1, 1, 1, 1, 1;
                1, 1, 1, 1, 1;
                1, 1, 1, 1, 1;
                1, 1, 1, 1, 1;
            ]
        );
    }

    #[test]
    fn test_identity() {
        let matrix = Matrix::identity(4);
        assert_eq!(
            matrix,
            rm![
                1, 0, 0, 0;
                0, 1, 0, 0;
                0, 0, 1, 0;
                0, 0, 0, 1;
            ]
        );
    }

    #[test]
    fn test_empty() {
        let matrix = Matrix::<Rational64>::empty();

        assert_eq!(matrix, Matrix::zeros((0, 0)));
        assert_eq!(matrix, Matrix::zeros((0, 7)));
        assert_eq!(matrix, Matrix::zeros((2, 0)));

        assert_eq!(matrix, Matrix::new_unsafe(vec![]));
        assert_eq!(matrix, Matrix::new_unsafe(vec![vec![]]));
    }

    #[test]
    fn test_simple_addition() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2, 3; 4, 5, 6];

        let result = m + n;
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{ccc}2 & 4 & 6\\8 & 10 & 12\end{array}\right]"
        );
    }

    #[test]
    fn test_simple_subtraction() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2, 3; 4, 5, 6];

        let result = m - n;
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{ccc}0 & 0 & 0\\0 & 0 & 0\end{array}\right]"
        );
    }

    #[test]
    fn test_simple_negation() {
        let m = im![1, 2, 3; 4, 5, 6];

        let result = -m;
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{ccc}-1 & -2 & -3\\-4 & -5 & -6\end{array}\right]"
        );
    }

    #[test]
    fn test_simple_multiplication() {
        let m = im![1, 2, 3; 4, 5, 6];
        let n = im![1, 2; 3, 4; 5, 6];

        let result = m * n;
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{cc}22 & 28\\49 & 64\end{array}\right]"
        );
    }

    #[test]
    fn test_simple_multiplication_with_scalar() {
        let m = im![1, 2, 3; 4, 5, 6];

        let result = m * 2;
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{ccc}2 & 4 & 6\\8 & 10 & 12\end{array}\right]"
        );
    }

    #[test]
    fn test_simple_multiplication_with_rational() {
        let m = rm![1, 2, 3; 4, 5, 6];

        let result = m * ri!(2);
        assert_eq!(
            result.to_latex(),
            r"\left[\begin{array}{ccc}2 & 4 & 6\\8 & 10 & 12\end{array}\right]"
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
        assert_eq!(m2.checked_pow(3).unwrap(), m2.clone() * m2.clone() * m2);
    }

    #[test]
    fn test_from_vec() {
        let m = Matrix::from_vec(rv![1, 2, 3, 4, 5, 6], (2, 3)).expect("Failed to create matrix");
        assert_eq!(
            m,
            rm![
                1, 2, 3;
                4, 5, 6;
            ]
        );

        let m = Matrix::from_vec(vec![1, 2, 3, 4, 5, 6], (2, 3)).expect("Failed to create matrix");
        assert_eq!(
            m,
            im![
                1, 2, 3;
                4, 5, 6;
            ]
        );
    }

    #[test]
    fn test_reshape() {
        let m = im![1, 2, 3, 4, 5, 6];
        let reshape = |m: &Matrix<i64>, shape| m.reshape(shape).expect("Failed to reshape matrix");
        assert_eq!(reshape(&m, (2, 3)), im![1, 2, 3; 4, 5, 6]);
        assert_eq!(reshape(&m, (3, 2)), im![1, 2; 3, 4; 5, 6]);
        assert_eq!(reshape(&m, (6, 1)), im![1; 2; 3; 4; 5; 6]);
        assert_eq!(reshape(&m, (1, 6)), im![1, 2, 3, 4, 5, 6]);
    }
}
