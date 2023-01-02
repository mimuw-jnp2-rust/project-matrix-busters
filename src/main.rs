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
use crate::parser::parse_expression;
use crate::traits::{GuiDisplayable, MatrixNumber};
use eframe::egui;
use egui::{Context, Sense, Ui};
use num_rational::Rational64;
use std::collections::HashMap;
use std::default::Default;

/// Field for matrices.
type K = Rational64;

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
    State {
        env,
        windows,
        shell: Default::default(),
    }
}

struct WindowState {
    is_open: bool,
}

#[derive(Default)]
struct ShellState {
    text: String,
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct State {
    env: Environment<K>,
    windows: HashMap<Identifier, WindowState>,
    shell: ShellState,
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

        display_shell::<K>(ctx, &mut self.state);

        // Center panel has to be added last, otherwise the side panel will be on top of it.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(get_translated(APP_NAME));
        });
    }
}

fn display_env_element(
    windows: &mut HashMap<Identifier, WindowState>,
    ui: &mut Ui,
    (identifier, value): (&Identifier, &mut Type<K>),
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
    (identifier, value): (&Identifier, &Type<K>),
    is_open: &mut bool,
) {
    egui::Window::new(identifier.to_string())
        .default_width(320.0)
        .open(is_open)
        .show(ctx, |ui| {
            ui.label(value.to_string());
        });
}

fn display_shell<T: MatrixNumber + ToString>(
    ctx: &Context,
    State {
        shell,
        env,
        windows,
    }: &mut State,
) {
    egui::TopBottomPanel::bottom("shell")
        .resizable(false)
        .default_height(128.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut shell.text)
                        .desired_rows(1)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
            let button_sense = if shell.text.is_empty() {
                Sense::hover()
            } else {
                Sense::click()
            };
            if ui
                .add(egui::Button::new(get_translated("Run")).sense(button_sense))
                .clicked()
            {
                match parse_shell_input(&shell.text, env) {
                    Ok((identifier, value)) => {
                        println!("{} = {}", identifier.to_string(), value.to_string());
                        shell.text.clear();
                        insert_to_env(env, identifier, value, windows);
                    }
                    Err(error) => {
                        println!("{}", error);
                        ui.label(error.to_string());
                    }
                }
            }
        });
}

/// Parse the input of the shell.
/// The input is expected to be of the form `identifier := expression`.
/// The expression is parsed and evaluated, and the result is stored in the environment.
/// The identifier is returned together with the result.
/// # Errors
/// If the input is not of the expected form, an error is returned.
/// # Arguments
/// * `input` - The input of the shell.
/// * `env` - The environment in which the expression is evaluated.
/// # Examples
/// ```
/// let mut env = Environment::new();
/// let input = "a := 1/2";
/// let (identifier, value) = parse_shell_input(input, &mut env).unwrap();
/// assert_eq!(identifier.to_string(), "a");
/// assert_eq!(value, Type::Rational(Rational64::new(1, 2)));
/// ```
fn parse_shell_input<T: MatrixNumber>(
    input: &str,
    env: &mut Environment<T>,
) -> anyhow::Result<(Identifier, Type<T>)> {
    let (identifier, expression) = input.split_once(":=").ok_or_else(|| {
        anyhow::anyhow!("Invalid input. Expected form `identifier := expression`.")
    })?;
    let identifier = Identifier::new(identifier.trim().to_string())?;
    let expression = parse_expression(expression.trim(), env)?;
    Ok((identifier, expression))
}

fn insert_to_env<T: MatrixNumber>(
    env: &mut Environment<T>,
    identifier: Identifier,
    value: Type<T>,
    windows: &mut HashMap<Identifier, WindowState>,
) {
    env.insert(identifier.clone(), value);
    windows.insert(identifier, WindowState { is_open: false });
}
