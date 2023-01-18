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
use crate::matrices::Matrix;
use crate::parser::parse_expression;
use crate::traits::{GuiDisplayable, MatrixNumber};
use anyhow::bail;
use eframe::egui;
use egui::{Context, Sense, Ui, Direction};
use num_rational::Rational64;
use std::collections::HashMap;
use std::default::Default;
use std::time::Duration;
use crate::locale::Language::*;
use crate::locale::Locale;

/// Field for matrices.
type K = Rational64;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
        ..Default::default()
    };
    let locale = Locale::new(Polish);
    eframe::run_native(
        &locale.get_translated(APP_NAME),
        options,
        Box::new(|_cc| Box::<MatrixApp>::new(MatrixApp::new(locale))),
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
        editor: Default::default(),
        toasts: egui_toast::Toasts::default(),
    }
}

struct WindowState {
    is_open: bool,
}

enum EditorType {
    Matrix(usize, usize, Vec<K>, String),
    Scalar(K, String),
}

#[derive(Default)]
struct EditorState {
    editor_type: Option<EditorType>,
}

#[derive(Default)]
struct ShellState {
    text: String,
}

#[derive(Default)]
struct State {
    env: Environment<K>,
    windows: HashMap<Identifier, WindowState>,
    shell: ShellState,
    editor: EditorState,
    toasts: egui_toast::Toasts,
}

struct MatrixApp {
    state: State,
    locale: Locale,
}

impl MatrixApp {
    fn new(locale: Locale) -> Self {
        Self {
            // state: State::default()
            state: mock_state(),
            locale,
        }
    }

    // Get Translated
    fn gt(&self, str: &str) -> String {
        self.locale.get_translated(str)
    }

    // Get Translated String
    fn gts(&self, str: String) -> String {
        self.locale.get_translated_from(str)
    }
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let window_size = frame.info().window_info.size;
        self.state.toasts = egui_toast::Toasts::default()
            .anchor((window_size.x - 10., window_size.y - 40.))
            .direction(egui::Direction::BottomUp)
            .align_to_end(true);

        display_menu_bar(ctx, &mut self.state, &self.locale);
        display_editor(ctx, &mut self.state, &self.locale);

        egui::SidePanel::left("objects")
            .resizable(true)
            .default_width(DEFAULT_LEFT_PANEL_WIDTH)
            .show(ctx, |ui| {
                egui::trace!(ui);
                ui.vertical_centered(|ui| {
                    ui.heading(self.gt("objects"));
                });

                ui.separator();

                self.state.env.iter_mut().for_each(|element| {
                    ui.horizontal(|ui| {
                        display_env_element(&mut self.state.windows, ui, element, &self.locale);
                    });
                });
            });

        self.state.windows.iter_mut().for_each(|(id, window)| {
            if window.is_open {
                let element = self.state.env.get(id).unwrap();
                display_env_element_window(ctx, (id, element), &mut window.is_open);
            }
        });

        display_shell::<K>(ctx, &mut self.state, &self.locale);

        // Center panel has to be added last, otherwise the side panel will be on top of it.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.gt(APP_NAME));
        });

        self.state.toasts.show(ctx);
    }
}

fn display_menu_bar(ctx: &Context, state: &mut State, locale: &Locale) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            display_add_matrix_button(ui, state, locale);
            display_add_scalar_button(ui, state, locale);
        })
    });
}

fn display_add_matrix_button(ui: &mut Ui, state: &mut State, locale: &Locale) {
    if ui.button(locale.get_translated("Add Matrix")).clicked() {
        const DEFAULT_ROWS: usize = 2;
        const DEFAULT_COLS: usize = 3;
        state.editor.editor_type = Some(EditorType::Matrix(
            DEFAULT_ROWS,
            DEFAULT_COLS,
            Matrix::zeros((DEFAULT_ROWS, DEFAULT_COLS))
                .consume()
                .into_iter()
                .flatten()
                .collect(),
            "".to_string(),
        ));
    }
}

fn display_add_scalar_button(ui: &mut Ui, state: &mut State, locale: &Locale) {
    if ui.button(locale.get_translated("Add Scalar")).clicked() {
        state.editor.editor_type = Some(EditorType::Scalar(K::default(), "".to_string()));
    }
}

fn display_editor(ctx: &Context, state: &mut State, locale: &Locale) {
    let mut editor_opened = state.editor.editor_type.is_some();
    let mut handled = false;
    egui::Window::new(locale.get_translated("Add new Matrix"))
        .open(&mut editor_opened)
        .show(ctx, |ui| {
            if let Some(editor_type) = &mut state.editor.editor_type {
                handled = match editor_type {
                    EditorType::Matrix(h, w, m, name) => display_matrix_editor(
                        ui,
                        &mut state.env,
                        &mut state.windows,
                        (h, w, m, name),
                        locale
                    ),
                    EditorType::Scalar(v, name) => {
                        display_scalar_editor(ui, &mut state.env, &mut state.windows, (v, name), locale)
                    }
                }
            }
        });
    if !editor_opened || handled {
        state.editor.editor_type = None;
    }
}

fn display_matrix_editor(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (h, w, data, name): (&mut usize, &mut usize, &mut Vec<K>, &mut String),
    locale: &Locale,
) -> bool {
    ui.label("Identifier:");
    ui.text_edit_singleline(name);
    ui.label(locale.get_translated("Enter matrix in the following format:"));
    ui.label("Height:");
    ui.add(egui::DragValue::new(h));
    ui.label("Width:");
    ui.add(egui::DragValue::new(w));
    if data.len() != *h * *w {
        data.resize(*h * *w, K::default());
    }

    egui::Grid::new("matrix_editor").show(ui, |ui| {
        for i in 0..*h {
            for j in 0..*w {
                ui.label(format!("({}, {})", i, j));
                display_k_editor((i, j), data, ui, *w);
            }
            ui.end_row();
        }
    });

    let can_save = !name.is_empty() && Identifier::is_valid(name);
    let button_sense = if can_save {
        Sense::click()
    } else {
        Sense::hover()
    };
    let mut handled = false;
    ui.horizontal(|ui| {
        let add_button = ui.add(egui::Button::new(locale.get_translated("Add")).sense(button_sense));
        if !can_save {
            ui.label(locale.get_translated("Identifier is invalid!"));
        }

        if add_button.clicked() {
            match Identifier::new(name.clone()) {
                Ok(identifier) => {
                    let value = Type::Matrix(Matrix::from_vec(data.clone(), (*h, *w)).unwrap());
                    insert_to_env(env, identifier, value, windows);
                    handled = true;
                }
                Err(_) => handled = false,
            }
        }
    });
    handled
}

fn display_k_editor((i, j): (usize, usize), data: &mut Vec<K>, ui: &mut Ui, width: usize) -> bool {
    let id = i * width + j;
    let mut value = data[id].to_string();
    ui.add(egui::TextEdit::singleline(&mut value));
    match value.parse() {
        Ok(parsed) => {
            data[id] = parsed;
            true
        }
        Err(_) => false,
    }
}

// TODO: refactor this function with `display_matrix_editor`
fn display_scalar_editor(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (v, name): (&mut K, &mut String),
    locale: &Locale,
) -> bool {
    ui.label("Identifier:");
    ui.text_edit_singleline(name);
    ui.label(locale.get_translated("Enter value in the following format:"));

    let mut numerator = v.numer().to_string();
    let mut denominator = v.denom().to_string();
    ui.horizontal(|ui| {
        ui.label("Nominator:");
        ui.add(egui::TextEdit::singleline(&mut numerator));
        ui.label("Denominator:");
        ui.add(egui::TextEdit::singleline(&mut denominator));
    });
    let parse = || -> anyhow::Result<K> {
        let numerator = numerator.parse()?;
        let denominator = denominator.parse()?;
        if denominator == 0 {
            bail!("Denominator cannot be 0!")
        } else {
            Ok(K::new(numerator, denominator))
        }
    };
    let new_value = parse();
    if new_value.is_ok() {
        *v = new_value.unwrap();
    }
    let can_save = !name.is_empty() && Identifier::is_valid(name);
    let button_sense = if can_save {
        Sense::click()
    } else {
        Sense::hover()
    };
    let mut handled = false;
    ui.horizontal(|ui| {
        let add_button = ui.add(egui::Button::new(locale.get_translated("Add")).sense(button_sense));
        if !can_save {
            ui.label(locale.get_translated("Identifier is invalid!"));
        }

        if add_button.clicked() {
            match Identifier::new(name.clone()) {
                Ok(identifier) => {
                    let value = Type::Scalar(*v);
                    insert_to_env(env, identifier, value, windows);
                    handled = true;
                }
                Err(_) => handled = false,
            }
        }
    });
    handled
}

fn display_env_element(
    windows: &mut HashMap<Identifier, WindowState>,
    ui: &mut Ui,
    (identifier, value): (&Identifier, &mut Type<K>),
    locale: &Locale,
) {
    let mut is_open = windows.get(identifier).unwrap().is_open;
    ui.horizontal(|ui| {
        ui.checkbox(&mut is_open, identifier.to_string());
        ui.label(value.display_string(locale));
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
        toasts,
        ..
    }: &mut State,
    locale: &Locale,
) {
    let mut run_shell_command = |shell_text: &mut String| {
        match parse_shell_input(shell_text, env) {
            Ok((identifier, value)) => {
                println!("{} = {}", identifier.to_string(), value.to_string());
                shell_text.clear();
                insert_to_env(env, identifier, value, windows);
            }
            Err(error) => {
                println!("{}", error);
                toasts.error(error.to_string(), Duration::from_secs(5));
            }
        }
    };

    egui::TopBottomPanel::bottom("shell")
        .resizable(false)
        .default_height(128.0)
        .show(ctx, |ui| {
            let button_sense = if shell.text.is_empty() {
                Sense::hover()
            } else {
                Sense::click()
            };

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    if ui
                    .add(egui::Button::new(locale.get_translated("Run")).sense(button_sense))
                    .clicked()
                    {
                        run_shell_command(&mut shell.text);
                    }

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut shell.text)
                            .desired_rows(1)
                            .desired_width(ui.available_width())
                            .code_editor(),
                    );
                    if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        run_shell_command(&mut shell.text);
                        response.request_focus();
                    }
                });
            });
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
