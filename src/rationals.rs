use std::fmt::Formatter;
use super::latex::LaTeXable;

/// This struct represents the rational number
/// `n/d` where `n` is the numerator and `d` is the denominator.
/// The sign of the rational number is always stored in the numerator.
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

    fn new(numerator: i64, denominator: i64) -> Rational {
        let sign = denominator.signum();
        let mut r = Rational { numerator: numerator * sign, denominator: denominator.unsigned_abs() };
        r.normalize();
        r
    }

    fn from_integer(n: i64) -> Rational {
        Rational { numerator: n, denominator: 1 }
    }
}

impl LaTeXable for Rational {
    fn to_latex(&self) -> String {
        if self.denominator == 1 {
            format!("{}", self.numerator)
        } else {
            format!("\\frac{{{}}}{{{}}}", self.numerator, self.denominator)
        }
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
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

    // TODO: Add more tests
    // TODO: Add tests for LaTeXable
    // TODO: Add tests for Display
    // TODO: Add tests for Add, Sub, Mul, Div, Neg, etc.
}
