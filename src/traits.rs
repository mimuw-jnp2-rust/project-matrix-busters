use eframe::epaint::TextShape;
use egui::{pos2, Color32, FontId, Shape};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, FromPrimitive, Signed, ToPrimitive,
};
use std::str::FromStr;

use crate::locale::Locale;
use num_traits::Num;

pub trait LaTeXable {
    fn to_latex(&self) -> String;
    fn to_latex_single(&self) -> String {
        self.to_latex()
    }
}

pub trait CheckedOps: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

impl<T> CheckedOps for T where T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv {}

pub trait GuiDisplayable {
    fn display_string(&self, locale: &Locale) -> String;

    fn to_shape(&self, ctx: &egui::Context, font_id: FontId, color: Color32) -> Shape;
}

pub trait BoxedShape {
    fn get_rect(&self) -> egui::Rect;
}

impl BoxedShape for Shape {
    fn get_rect(&self) -> egui::Rect {
        match self {
            Shape::Text(text_shape) => text_shape.galley.rect,
            _ => self.visual_bounding_rect(),
        }
    }
}

pub trait MatrixNumber:
    Num
    + CheckedOps
    + FromPrimitive
    + ToPrimitive
    + Signed
    + LaTeXable
    + GuiDisplayable
    + Clone
    + FromStr
    + ToString
{
}

impl<T> MatrixNumber for T where
    T: Num
        + CheckedOps
        + FromPrimitive
        + ToPrimitive
        + Signed
        + LaTeXable
        + GuiDisplayable
        + Clone
        + FromStr
        + ToString
{
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

#[macro_export]
macro_rules! gui_displayable_for_primitive {
    ($($t:ty),*) => {
        $(
            impl GuiDisplayable for $t {
                fn display_string(&self, _locale: &Locale) -> String {
                    self.to_string()
                }

                fn to_shape(&self, ctx: &egui::Context, font_id: FontId, color: Color32) -> Shape {
                    let text_shape = TextShape::new(
                        pos2(0., 0.),
                        ctx.fonts().layout_no_wrap(self.to_string(), font_id, color),
                    );
                    Shape::Text(text_shape)
                }
            }
        )*
    }
}

// We add LaTeX support for all the basic types
to_string_to_latex!(i8, i16, i32, i64, i128, isize);

// We add display support for all the basic types
gui_displayable_for_primitive!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_number() {
        fn test<T: MatrixNumber>() {
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
