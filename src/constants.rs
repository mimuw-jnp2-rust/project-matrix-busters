pub const APP_NAME: &str = "JP2GMD - Matrix Calculator";
pub const DEFAULT_WIDTH: f32 = 800.0;
pub const DEFAULT_HEIGHT: f32 = 600.0;
pub const DEFAULT_LEFT_PANEL_WIDTH: f32 = DEFAULT_WIDTH * 0.20;

pub const TEXT_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;
pub const FONT_ID: egui::FontId = egui::FontId::proportional(18.);

pub const MATRIX_HPADDING: f32 = 15.;
pub const MATRIX_VPADDING: f32 = 8.;
pub const FRACTION_FONT_SIZE_RATIO: f32 = 0.8;
pub const FRACTION_HMARGIN: f32 = 2.;
pub const FRACTION_VMARGIN: f32 = 1.;
pub const FRACTION_LINE_WIDTH: f32 = 1.;
pub const VALUE_PADDING: f32 = 15.;

pub const FLOAT_STRING_PRECISION: usize = 3;

pub const ICON_PATH: &str = "assets/icon.png";
#[cfg(feature = "fft")]
pub const DFT_PATH: &str = "assets/dft_result.json";
