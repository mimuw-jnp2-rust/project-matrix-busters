use crate::matrices::Matrix;
use crate::traits::{LaTeXable, MatrixNumber};
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Aftermath<T: MatrixNumber> {
    pub result: Matrix<T>,
    pub steps: Vec<String>,
}

#[allow(dead_code)]
impl<T: MatrixNumber> Matrix<T> {
    /// Returns a copy of the matrix which is in the row echelon form along
    /// with all steps represented in human-friendly LaTeX notation.
    /// Uses Gaussian elimination combined with some heuristics aiming at
    /// making the number of steps as small as possible.
    pub fn echelon(&self) -> anyhow::Result<Aftermath<T>> {
        const CONTEXT: &str = "Calculations error!";

        if self.is_empty() {
            return Ok(Aftermath {
                result: self.clone(),
                steps: vec![],
            });
        }

        let (rows, cols) = self.get_shape();

        let mut steps = vec![self.to_latex()];
        let mut c = 0;
        let mut i = 0;
        let mut data = self.deep_matrix_data_clone();

        while c < cols && i < rows {
            let mut j = i;
            for k in i + 1..rows {
                if Self::nice(&data[k][c]).context(CONTEXT)?
                    < Self::nice(&data[j][c]).context(CONTEXT)?
                {
                    j = k;
                }
            }

            if !data[j][c].is_zero() {
                if i != j {
                    data.swap(i, j);
                    data = Self::push_step(
                        &mut steps,
                        format!(r"w_{{{}}} \leftrightarrow w_{{{}}}", i + 1, j + 1).as_str(),
                        data,
                    );
                }

                if !data[i][c].is_one() {
                    let d = data[i][c].clone();
                    for j in c..cols {
                        data[i][j] = data[i][j].checked_div(&d).context(CONTEXT)?;
                    }

                    data = Self::push_step(
                        &mut steps,
                        format!(r"w_{{{}}} : {}", i + 1, d.to_latex_single()).as_str(),
                        data,
                    );
                }

                let mut step_ops: Vec<String> = Vec::new();
                for j in 0..rows {
                    if j != i && !data[j][c].is_zero() {
                        let p = data[j][c].checked_div(&data[i][c]).context(CONTEXT)?;
                        for k in c..cols {
                            data[j][k] = data[j][k]
                                .checked_sub(&data[i][k].checked_mul(&p).context(CONTEXT)?)
                                .context(CONTEXT)?;
                        }

                        step_ops.push(format!(
                            "w_{{{}}} {}w_{{{}}}",
                            j + 1,
                            Self::sub_coefficient_to_latex(&p).context(CONTEXT)?,
                            i + 1
                        ));
                    }
                }

                if !step_ops.is_empty() {
                    data = Self::push_step(
                        &mut steps,
                        format!(r"\substack{{{}}}", &step_ops.join(r"\\")).as_str(),
                        data,
                    );
                }

                i += 1;
            }

            c += 1;
        }

        Ok(Aftermath {
            result: Self::new_unsafe(data),
            steps,
        })
    }

    /// Returns a deep copy of matrix data vector.
    fn deep_matrix_data_clone(&self) -> Vec<Vec<T>> {
        self.get_data().iter().map(|row| row.to_vec()).collect()
    }

    /// Inserts the LaTeX representation of a single echelonization step with
    /// transitions `transitions` and new matrix containing `data` into `steps`.
    /// Returns unchanged `data`.
    fn push_step(steps: &mut Vec<String>, transitions: &str, data: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let temp_matrix = Matrix::new_unsafe(data);
        steps.push(format!(
            r"\xrightarrow{{{}}} {}",
            transitions,
            temp_matrix.to_latex()
        ));
        temp_matrix.consume()
    }

    /// Returns an integer representing how nice a row starting with the given
    /// coefficient is to be used in a step of Gaussian elimination. The smaller
    /// value means the better choice.
    fn nice(coefficient: &T) -> Option<i64> {
        if coefficient.is_zero() {
            // should not be chosen if there is any row with nonzero leading coefficient
            Some(1000)
        } else if coefficient.is_one() {
            // the easiest one, does not need to be changed
            Some(0)
        } else if (T::zero().checked_sub(coefficient)?).is_one() {
            // we only have to negate all elements
            Some(1)
        } else {
            // if there is no better choice...
            Some(2)
        }
    }

    /// How the substraction of an identifier multiplied by the given coefficient
    /// should be printed in LaTeX.
    /// For example, `sub_coefficient_to_latex(1)` returns `- `, as it is not
    /// necessary to write `- 1w`, but `- w` is sufficient and
    /// `sub_coefficient_to_latex(-5)` returns `+ 5`, because `+ 5w` is easier
    /// to read than `- (-5)w`.
    /// Assumes that the coefficient is nonzero.
    fn sub_coefficient_to_latex(coefficient: &T) -> Option<String> {
        if coefficient.is_one() {
            Some("- ".to_string())
        } else if (T::zero().checked_sub(coefficient)?).is_one() {
            Some("+ ".to_string())
        } else if coefficient.is_positive() {
            Some(format!("- {}", coefficient.to_latex()))
        } else if coefficient.is_negative() {
            Some(format!(
                "+ {}",
                (T::zero().checked_sub(coefficient)?.to_latex())
            ))
        } else {
            unreachable!("Should not be used for zero coefficient!")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::LaTeXable;
    use crate::{matrices::Matrix, ri, rm, rv};
    use num_rational::Rational64;

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
}
