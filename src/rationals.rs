use std::fmt::Formatter;
use super::latex::LaTeXable;

/// This struct represents the rational number
/// `n/d` where `n` is the numerator and `d` is the denominator.
/// The sign of the rational number is always stored in the numerator.
#[derive(Debug)]
struct Rational {
    numerator: i64,
    denominator: u64,
}

impl Rational {
    fn normalize(&mut self) {
        let gcd = gcd(self.numerator.unsigned_abs(), self.denominator);
        self.numerator /= gcd as i64;
        self.denominator /= gcd;
    }

    fn new_non_signed_denominator(numerator: i64, denominator: u64) -> Rational {
        let mut r = Rational { numerator, denominator };
        r.normalize();
        r
    }

    pub fn new(numerator: i64, denominator: i64) -> Rational {
        let sign = denominator.signum();
        let mut r = Rational { numerator: numerator * sign, denominator: denominator.unsigned_abs() };
        r.normalize();
        r
    }

    pub fn from_integer(n: i64) -> Rational {
        Rational { numerator: n, denominator: 1 }
    }

    fn cmp(&self, other: &Rational) -> std::cmp::Ordering {
        let a = self.numerator * other.denominator as i64;
        let b = other.numerator * self.denominator as i64;
        a.cmp(&b)
    }
}

impl LaTeXable for Rational {
    fn to_latex(&self) -> String {
        let sign = if self.numerator < 0 { "-" } else { "" };
        match self.denominator {
            1 => format!("{}", self.numerator),
            _ => format!("{}\\frac{{{}}}{{{}}}", sign, self.numerator.unsigned_abs(), self.denominator),
        }
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.denominator {
            1 => write!(f, "{}", self.numerator),
            _ => write!(f, "{}/{}", self.numerator, self.denominator),
        }
    }
}

impl std::ops::Add for Rational {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new_non_signed_denominator(
            self.numerator * other.denominator as i64 + other.numerator * self.denominator as i64,
            self.denominator * other.denominator
        )
    }
}

impl std::ops::Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new_non_signed_denominator(-self.numerator, self.denominator)
    }
}

impl std::ops::Sub for Rational {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl std::ops::Mul for Rational {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new_non_signed_denominator(
            self.numerator * other.numerator,
            self.denominator * other.denominator
        )
    }
}

impl std::ops::Div for Rational {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let sign = other.numerator.signum();
        Self::new_non_signed_denominator(
            self.numerator * other.denominator as i64 * sign,
            self.denominator * other.numerator.unsigned_abs() as u64
        )
    }
}

impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        self.numerator == other.numerator && self.denominator == other.denominator
    }
}

impl Eq for Rational {}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp(other)
    }
}

/// This function computes the greatest common divisor of two numbers.
/// It uses the Euclidean algorithm.
/// # Arguments
/// * `a` - The first number
/// * `b` - The second number
/// # Returns
/// The greatest common divisor of `a` and `b`.
/// # Examples
/// ```
/// let gcd = gcd(42, 56);
/// assert_eq!(gcd, 14);
/// ```
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(2, 4), 2);
        assert_eq!(gcd(36, 6), 6);

        assert_eq!(gcd(0, 0), 0);
        assert_eq!(gcd(0, 1), 1);
        assert_eq!(gcd(1, 0), 1);

        assert_eq!(gcd(0, 15), 15);
        assert_eq!(gcd(16, 0), 16);

        assert_eq!(gcd(2137, 2137), 2137);
        assert_eq!(gcd(42, 56), 14);
    }

    #[test]
    fn test_latexable() {
        let latex_test = |numerator, denominator, expected| {
            let r = Rational::new(numerator, denominator);
            assert_eq!(r.to_latex(), expected);
        };

        latex_test(1, 1, "1");
        latex_test(1, 2, "\\frac{1}{2}");
        latex_test(1, -2, "-\\frac{1}{2}");
        latex_test(-1, 2, "-\\frac{1}{2}");
        latex_test(-1, -2, "\\frac{1}{2}");
        latex_test(36, 15, "\\frac{12}{5}");
    }

    #[test]
    fn test_from_integer() {
        let integer_test = |num, latex| {
            let n = Rational::from_integer(num);
            assert_eq!(n.to_string(), latex);
        };

        integer_test(0, "0");
        integer_test(1, "1");
        integer_test(-1, "-1");
        integer_test(42, "42");
    }

    #[test]
    fn test_add_rationals() {
        let test_add = |a: (i64, i64), b: (i64, i64), expected: (i64, i64)| {
            let a = Rational::new(a.0, a.1);
            let b = Rational::new(b.0, b.1);
            let expected = Rational::new(expected.0, expected.1);
            assert_eq!(a + b, expected);
        };

        test_add((1, 1), (1, 1), (2, 1));
        test_add((1, 2), (1, 2), (1, 1));
        test_add((1, 2), (1, 3), (5, 6));
        test_add((1, 2), (1, 4), (3, 4));
        test_add((1, 2), (1, 6), (2, 3));
        test_add((1, 2), (1, 8), (5, 8));
        test_add((1, 2), (1, 10), (6, 10));
        test_add((1, 2), (1, 12), (7, 12));
        test_add((1, 2), (1, 14), (8, 14));

        test_add((-1, 2), (1, 2), (0, 1));
        test_add((-1, 3), (-1, 4), (-7, 12));
        test_add((-1, 4), (-1, 6), (-5, 12));
    }

    #[test]
    fn test_sub_rationals() {
        let test_sub = |a: (i64, i64), b: (i64, i64), expected: (i64, i64)| {
            let a = Rational::new(a.0, a.1);
            let b = Rational::new(b.0, b.1);
            let expected = Rational::new(expected.0, expected.1);
            assert_eq!(a - b, expected);
        };

        test_sub((1, 1), (1, 1), (0, 1));
        test_sub((1, 2), (1, 2), (0, 1));
        test_sub((1, 2), (1, 3), (1, 6));
        test_sub((1, 2), (1, 4), (1, 4));

        test_sub((5, 12), (7, 36), (2, 9));

    }

    // TODO: Add tests for Add, Sub, Mul, Div, Neg, etc.
}
