use crate::traits::LaTeXable;
use num_rational::Rational64;
use num_traits::sign::Signed;

impl LaTeXable for Rational64 {
    fn to_latex(&self) -> String {
        match self.is_integer() {
            true => format!("{}", self.numer()),
            false => format!(
                "{}\\frac{{{}}}{{{}}}",
                if self.is_positive() { "" } else { "-" },
                self.numer().unsigned_abs(),
                self.denom().unsigned_abs()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::MatrixNumber;
    use num_rational::Rational64;

    #[test]
    fn test_simple_fraction() {
        let r = Rational64::new(7, 21);
        assert_eq!(r.to_latex(), "\\frac{1}{3}");
    }

    #[test]
    fn test_negative_fraction() {
        let r = Rational64::new(-1 * 42, 2 * 42);
        assert_eq!(r.to_latex(), "-\\frac{1}{2}");
    }

    #[test]
    fn test_negative_denominator() {
        let r = Rational64::new(1 * 151, -2 * 151);
        assert_eq!(r.to_latex(), "-\\frac{1}{2}");
    }

    #[test]
    fn test_negative_numerator_and_denominator() {
        let r = Rational64::new(-10, -20);
        assert_eq!(r.to_latex(), "\\frac{1}{2}");
    }

    #[test]
    fn test_fraction_normalization() {
        let r = Rational64::new(4, 1);
        assert_eq!(r.to_latex(), "4");
    }

    #[test]
    fn test_fraction_zero() {
        let r = Rational64::new(0, 1);
        assert_eq!(r.to_latex(), "0");
    }

    #[test]
    fn test_matrix_num() {
        fn test<T: MatrixNumber>(_: T) {}

        let r = Rational64::new(4, 1);
        test(r);
    }
}
