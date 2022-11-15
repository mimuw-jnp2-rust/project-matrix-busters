use crate::traits::LaTeXable;
use crate::traits::MatrixNumber;
use anyhow;

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

impl LaTeXable for i32 {
    fn to_latex(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::LaTeXable;

    #[test]
    fn test_matrix() {
        let matrix = super::Matrix::<i32>::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(
            matrix.to_latex(),
            r"\begin{bmatrix}1 & 2 & 3\\4 & 5 & 6\end{bmatrix}"
        );
    }
}
