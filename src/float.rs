use crate::constants::FLOAT_STRING_PRECISION;
use crate::locale::Locale;
use crate::traits::{GuiDisplayable, LaTeXable};
use eframe::epaint::{Color32, FontId, Shape, TextShape};
use egui::{pos2, Context};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, FromPrimitive, Num, One, Signed, ToPrimitive,
    Zero,
};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct Float64 {
    value: f64,
}

impl Num for Float64 {
    type FromStrRadixErr = <f64 as Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        <f64 as Num>::from_str_radix(str, radix).map(|v| v.into())
    }
}

impl PartialEq for Float64 {
    fn eq(&self, other: &Self) -> bool {
        <f64 as PartialEq>::eq(&self.value, &other.value)
    }
}

impl From<Float64> for f64 {
    fn from(value: Float64) -> Self {
        value.value
    }
}

impl From<f64> for Float64 {
    fn from(value: f64) -> Self {
        Float64 { value }
    }
}

impl Zero for Float64 {
    fn zero() -> Self {
        <f64 as Zero>::zero().into()
    }

    fn is_zero(&self) -> bool {
        <f64 as Zero>::is_zero(&self.value)
    }
}

impl Add<Self> for Float64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.value.add(rhs.value).into()
    }
}

impl One for Float64 {
    fn one() -> Self {
        <f64 as One>::one().into()
    }
}

impl Mul<Self> for Float64 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.value.mul(rhs.value).into()
    }
}

impl Sub<Self> for Float64 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.value.sub(rhs.value).into()
    }
}

impl Div<Self> for Float64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.value.div(rhs.value).into()
    }
}

impl Rem<Self> for Float64 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.value.rem(rhs.value).into()
    }
}

impl CheckedAdd for Float64 {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(self.value.add(v.value).into())
    }
}

impl CheckedSub for Float64 {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(self.value.sub(v.value).into())
    }
}

impl CheckedMul for Float64 {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(self.value.mul(v.value).into())
    }
}

impl CheckedDiv for Float64 {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_zero() {
            return None;
        }
        Some(self.value.div(v.value).into())
    }
}

impl FromPrimitive for Float64 {
    fn from_i64(n: i64) -> Option<Self> {
        Some((n as f64).into())
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some((n as f64).into())
    }
}

impl ToPrimitive for Float64 {
    fn to_i64(&self) -> Option<i64> {
        Some(self.value as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(self.value as u64)
    }
}

impl Signed for Float64 {
    fn abs(&self) -> Self {
        self.value.abs().into()
    }

    fn abs_sub(&self, other: &Self) -> Self {
        (self.value - other.value).abs().into()
    }

    fn signum(&self) -> Self {
        self.value.signum().into()
    }

    fn is_positive(&self) -> bool {
        self.value.is_sign_positive()
    }

    fn is_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

impl Neg for Float64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.value.neg().into()
    }
}

impl FromStr for Float64 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>().map(|v| v.into()).map_err(|_| ())
    }
}

impl ToString for Float64 {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl LaTeXable for Float64 {
    fn to_latex(&self) -> String {
        self.value.to_latex()
    }
}

impl GuiDisplayable for Float64 {
    fn display_string(&self, locale: &Locale) -> String {
        self.value.display_string(locale)
    }

    fn to_shape(&self, ctx: &Context, font_id: FontId, color: Color32) -> Shape {
        self.value.to_shape(ctx, font_id, color)
    }
}

impl LaTeXable for f64 {
    fn to_latex(&self) -> String {
        trim_trailing_zeros_float_str(&format!("{:.*}", FLOAT_STRING_PRECISION, self))
    }
}

/// Trims trailing zeros from a float string.
/// # Arguments
/// * `s` - String to trim zeros from
/// # Returns
/// A string representing the same floating point number, but without trailing zeros
/// # Examples
/// ```rust
/// # use crate::jp2gmd_lib::trim_trailing_zeros_float_str;
/// assert_eq!(trim_trailing_zeros_float_str("10.0"), "10");
/// assert_eq!(trim_trailing_zeros_float_str("123.450"), "123.45");
/// assert_eq!(trim_trailing_zeros_float_str("1.0"), "1");
/// ```
pub fn trim_trailing_zeros_float_str(s: &str) -> String {
    let mut s = s.to_string();
    if s.contains('.') {
        s = s.trim_end_matches('0').to_string();
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

impl GuiDisplayable for f64 {
    fn display_string(&self, _locale: &Locale) -> String {
        self.to_latex()
    }

    fn to_shape(&self, ctx: &Context, font_id: FontId, color: Color32) -> Shape {
        let text_shape = TextShape::new(
            pos2(0., 0.),
            ctx.fonts(|f| f.layout_no_wrap(self.to_latex(), font_id, color)),
        );
        Shape::Text(text_shape)
    }
}
