mod constants;
mod environment;
mod locale;
mod matrices;
mod matrix_algorithms;
mod parser;
mod rationals;
mod traits;

use crate::constants::{APP_NAME, DEFAULT_HEIGHT, DEFAULT_LEFT_PANEL_WIDTH, DEFAULT_WIDTH};
use crate::environment::{Environment, Identifier, Type};
use crate::locale::get_translated;
use crate::matrices::Matrix;
use crate::traits::GuiDisplayable;
use eframe::egui;
use egui::{Context, Ui};
use num_rational::Rational64;
use std::collections::HashMap;
use std::default::Default;

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

fn mock_state() -> State {
    let mut env = Environment::new();
    env.insert(
        Identifier::new("a".to_string()).unwrap(),
        Type::Scalar(Rational64::new(1, 2)),
    );
    env.insert(
        Identifier::new("M".to_string()).unwrap(),
        Type::Matrix(rm![1, 2; 3, 4]),
    );

    let mut windows = HashMap::new();
    windows.insert(
        Identifier::new("a".to_string()).unwrap(),
        WindowState { is_open: true },
    );
    windows.insert(
        Identifier::new("M".to_string()).unwrap(),
        WindowState { is_open: false },
    );
    State { env, windows }
}

struct WindowState {
    is_open: bool,
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct State {
    env: Environment<Rational64>,
    windows: HashMap<Identifier, WindowState>,
}

struct MatrixApp {
    state: State,
}

impl Default for MatrixApp {
    fn default() -> Self {
        Self {
            // state: State::default()
            state: mock_state(),
        }
    }
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("objects")
            .resizable(true)
            .default_width(DEFAULT_LEFT_PANEL_WIDTH)
            .show(ctx, |ui| {
                egui::trace!(ui);
                ui.vertical_centered(|ui| {
                    ui.heading(get_translated("objects"));
                });

                ui.separator();

                self.state.env.iter_mut().for_each(|element| {
                    ui.horizontal(|ui| {
                        display_env_element(&mut self.state.windows, ui, element);
                    });
                });
            });

        self.state.windows.iter_mut().for_each(|(id, window)| {
            if window.is_open {
                let element = self.state.env.get(id).unwrap();
                display_env_element_window(ctx, (id, element), &mut window.is_open);
            }
        });

        // Center panel has to be added last, otherwise the side panel will be on top of it.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(get_translated(APP_NAME));
        });
    }
}

fn display_env_element(
    windows: &mut HashMap<Identifier, WindowState>,
    ui: &mut Ui,
    (identifier, value): (&Identifier, &mut Type<Rational64>),
) {
    let mut is_open = windows.get(identifier).unwrap().is_open;
    ui.horizontal(|ui| {
        ui.checkbox(&mut is_open, identifier.to_string());
        ui.label(value.display_string());
    });
    windows.insert(identifier.clone(), WindowState { is_open });
}

fn display_env_element_window(
    ctx: &Context,
    (identifier, value): (&Identifier, &Type<Rational64>),
    is_open: &mut bool,
) {
    egui::Window::new(identifier.to_string())
        .default_width(320.0)
        .open(is_open)
        .show(ctx, |ui| {
            ui.label(value.to_string());
        });
}
