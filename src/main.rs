mod constants;
mod locale;
mod matrices;
mod rationals;
mod traits;

use crate::constants::{APP_NAME, DEFAULT_HEIGHT, DEFAULT_LEFT_PANEL_WIDTH, DEFAULT_WIDTH};
use crate::locale::get_translated;
use crate::matrices::Matrix;
use eframe::egui;
use num_rational::Rational64;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
        ..Default::default()
    };
    eframe::run_native(
        &get_translated(APP_NAME),
        options,
        Box::new(|_cc| Box::<MatrixApp>::default()),
    )
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct State {
    matrices: Vec<Matrix<Rational64>>,
}

struct MatrixApp {
    state: State,
}

impl Default for MatrixApp {
    fn default() -> Self {
        Self {
            // state: State::default(), TODO: restore
            state: State {
                matrices: vec![
                    rm![
                        1, 2, 3;
                        4, 5, 6;
                        7, 8, 9;
                    ],
                    rm![
                        4, 5;
                        6, 7;
                    ],
                ],
            },
        }
    }
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("objects")
            .resizable(true)
            .default_width(DEFAULT_LEFT_PANEL_WIDTH)
            .show(ctx, |ui| {
                egui::trace!(ui);
                ui.vertical_centered(|ui| {
                    ui.heading(get_translated("objects"));
                });

                ui.separator();

                self.state.matrices.iter_mut().for_each(|matrix| {
                    ui.horizontal(|ui| {
                        ui.label(get_translated("matrix"));
                        ui.add(egui::Label::new(matrix.to_string()));
                    });
                });
            });

        // Center panel has to be added last, otherwise the side panel will be on top of it.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(get_translated(APP_NAME));
        });
    }
}
