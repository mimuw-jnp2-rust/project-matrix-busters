use num_traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv};

use num_traits::Num;

pub trait LaTeXable {
    fn to_latex(&self) -> String;
}

pub trait CheckedOps : CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

impl<T> CheckedOps for T where T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

pub trait MatrixNumber : Num + CheckedOps + LaTeXable {}

impl<T> MatrixNumber for T where T: Num + CheckedOps + LaTeXable {}
