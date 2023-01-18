use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, FromPrimitive, Signed, ToPrimitive,
};
use std::ops::Mul;
use std::str::FromStr;

use num_traits::Num;
use crate::locale::Locale;

pub trait LaTeXable {
    fn to_latex(&self) -> String;
    fn to_latex_single(&self) -> String {
        self.to_latex()
    }
}

pub trait CheckedMulScl<T: MatrixNumber>: Sized + Mul<Self, Output = Self> {
    fn checked_mul_scl(&self, other: &T) -> Option<Self>;
}

pub trait CheckedOps: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

impl<T> CheckedOps for T where T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

pub trait MatrixNumber:
    Num + CheckedOps + FromPrimitive + ToPrimitive + Signed + LaTeXable + Clone + FromStr
{
}

impl<T> MatrixNumber for T where
    T: Num + CheckedOps + FromPrimitive + ToPrimitive + Signed + LaTeXable + Clone + FromStr
{
}

pub trait GuiDisplayable {
    fn display_string(&self, locale: &Locale) -> String;
}

#[macro_export]
macro_rules! to_string_to_latex {
    ($($t:ty),*) => {
        $(
            impl LaTeXable for $t {
                fn to_latex(&self) -> String {
                    self.to_string()
                }
            }
        )*
    }
}

// We add LaTeX support for all the basic types
to_string_to_latex!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_number() {
        fn test<T: MatrixNumber + ToString>() {
            let t = T::one();
            assert_eq!(t.to_latex(), t.to_string());
        }

        macro_rules! test_matrix_number {
            ($($t:ty),*) => {
                $(
                    test::<$t>();
                )*
            }
        }

        test_matrix_number!(i8, i16, i32, i64, i128, isize);
    }
}
