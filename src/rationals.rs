use crate::{
    constants::{
        FRACTION_FONT_SIZE_RATIO, FRACTION_HMARGIN, FRACTION_LINE_WIDTH, FRACTION_VMARGIN,
    },
    traits::{GuiDisplayable, LaTeXable},
};
use egui::{pos2, vec2, FontId, Rect, Rounding, Shape};
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

    fn to_latex_single(&self) -> String {
        if self.is_positive() {
            self.to_latex()
        } else {
            format!(r"\left({}\right)", self.to_latex())
        }
    }
}

impl GuiDisplayable for Rational64 {
    fn display_string(&self, _locale: &crate::locale::Locale) -> String {
        self.to_string()
    }

    fn to_shape(
        &self,
        ctx: &egui::Context,
        font_id: egui::FontId,
        color: egui::Color32,
    ) -> egui::Shape {
        if self.is_integer() {
            self.numer().to_shape(ctx, font_id, color)
        } else {
            let mut num_shape = self.numer().to_shape(
                ctx,
                FontId {
                    size: font_id.size * FRACTION_FONT_SIZE_RATIO,
                    family: font_id.family.clone(),
                },
                color,
            );
            let mut denom_shape = self.denom().to_shape(
                ctx,
                FontId {
                    size: font_id.size * FRACTION_FONT_SIZE_RATIO,
                    family: font_id.family,
                },
                color,
            );

            let num_rect = num_shape.visual_bounding_rect();
            let denom_rect = denom_shape.visual_bounding_rect();
            println!("{:?} {:?}", num_rect, denom_rect);
            let single_width = num_rect.width().max(denom_rect.width()) + 2. * FRACTION_HMARGIN;

            num_shape.translate(vec2((single_width - num_rect.width()) / 2., 0.));
            denom_shape.translate(vec2(
                (single_width - denom_rect.width()) / 2.,
                num_rect.height() + 2. * FRACTION_VMARGIN + FRACTION_LINE_WIDTH,
            ));

            let line_shape = Shape::rect_filled(
                Rect {
                    min: pos2(0., num_rect.height() + FRACTION_VMARGIN),
                    max: pos2(
                        single_width,
                        num_rect.height() + FRACTION_VMARGIN + FRACTION_LINE_WIDTH,
                    ),
                },
                Rounding::none(),
                color,
            );

            Shape::Vec(vec![num_shape, denom_shape, line_shape])
        }
    }
}

// Macro to generate a Rational64 from a integer.
// `ri!(1)` is equivalent to `Rational64::from_integer(1)`, but shorter.
// ri stands for Rational from Integer.
#[macro_export]
macro_rules! ri {
    ($($t:expr),*) => {
        $(
            Rational64::from_integer($t)
        )*
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
        let r = Rational64::new(-42, 2 * 42);
        assert_eq!(r.to_latex(), "-\\frac{1}{2}");
    }

    #[test]
    fn test_negative_denominator() {
        let r = Rational64::new(151, -2 * 151);
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
