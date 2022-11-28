use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

use num_traits::Num;

pub trait LaTeXable {
    fn to_latex(&self) -> String;
}

pub trait CheckedOps: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

impl<T> CheckedOps for T where T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

pub trait MatrixNumber: Num + CheckedOps + LaTeXable + Clone {}

impl<T> MatrixNumber for T where T: Num + CheckedOps + LaTeXable + Clone {}

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
to_string_to_latex!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

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

        test_matrix_number!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    }
}
