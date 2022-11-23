use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Signed};

use num_traits::Num;

pub trait LaTeXable {
    fn to_latex(&self) -> String;
    fn to_latex_single(&self) -> String;
}

pub trait CheckedOps: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

impl<T> CheckedOps for T where T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

pub trait MatrixNumber: Num + CheckedOps + Signed + LaTeXable + Clone {}

impl<T> MatrixNumber for T where T: Num + CheckedOps + Signed + LaTeXable + Clone {}
