mod constants;
mod editor_gui;
mod env_gui;
mod environment;
mod locale;
mod matrices;
mod matrix_algorithms;
mod parser;
mod rationals;
mod traits;

use crate::constants::{APP_NAME, DEFAULT_HEIGHT, DEFAULT_LEFT_PANEL_WIDTH, DEFAULT_WIDTH};
use crate::editor_gui::{display_editor, set_editor_to_matrix, set_editor_to_scalar, EditorState};
use crate::environment::{Environment, Identifier, Type};
use crate::locale::Language::*;
use crate::locale::Locale;
use crate::matrices::Matrix;
use crate::matrix_algorithms::Aftermath;
use crate::parser::parse_instruction;
use crate::traits::{GuiDisplayable, LaTeXable, MatrixNumber};
use clipboard::{ClipboardContext, ClipboardProvider};
use constants::{FONT_ID, TEXT_COLOR};
use eframe::egui;

use egui::{Context, Sense, Ui};
use num_rational::Rational64;
use std::collections::HashMap;
use std::default::Default;
use std::time::Duration;

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

fn mock_state() -> State<K> {
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
        clipboard: ClipboardProvider::new().expect("Failed to create Clipboard context!"),
    }
}

pub struct WindowState {
    is_open: bool,
}

#[derive(Default)]
struct ShellState {
    text: String,
}

pub struct State<K>
where
    K: MatrixNumber,
{
    env: Environment<K>,
    windows: HashMap<Identifier, WindowState>,
    shell: ShellState,
    editor: EditorState,
    toasts: egui_toast::Toasts,
    clipboard: ClipboardContext,
}

impl<K> Default for State<K>
where
    K: MatrixNumber,
{
    fn default() -> Self {
        Self {
            env: Default::default(),
            windows: Default::default(),
            shell: Default::default(),
            editor: Default::default(),
            toasts: Default::default(),
            clipboard: ClipboardContext::new().expect("Failed to create Clipboard context!"),
        }
    }
}

struct MatrixApp {
    state: State<K>,
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
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let window_size = frame.info().window_info.size;
        self.state.toasts = egui_toast::Toasts::default()
            .anchor((window_size.x - 10., window_size.y - 40.))
            .direction(egui::Direction::BottomUp)
            .align_to_end(true);

        display_menu_bar(ctx, &mut self.state, &self.locale);
        display_editor::<K>(ctx, &mut self.state, &self.locale);

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
                display_env_element_window(
                    ctx,
                    (id, element),
                    &mut self.state.clipboard,
                    &mut window.is_open,
                );
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

fn display_menu_bar(ctx: &Context, state: &mut State<K>, locale: &Locale) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            display_add_matrix_button(ui, state, locale);
            display_add_scalar_button(ui, state, locale);
        })
    });
}

fn display_add_matrix_button(ui: &mut Ui, state: &mut State<K>, locale: &Locale) {
    if ui.button(locale.get_translated("Add Matrix")).clicked() {
        set_editor_to_matrix(&mut state.editor, &K::default().to_string());
    }
}

fn display_add_scalar_button(ui: &mut Ui, state: &mut State<K>, locale: &Locale) {
    if ui.button(locale.get_translated("Add Scalar")).clicked() {
        set_editor_to_scalar(&mut state.editor, &K::default().to_string());
    }
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
    clipboard: &mut ClipboardContext,
    is_open: &mut bool,
) {
    egui::Window::new(identifier.to_string())
        .open(is_open)
        .show(ctx, |ui| {
            ui.allocate_space(ui.available_size());
            let mut value_shape = value.to_shape(ctx, FONT_ID, TEXT_COLOR);
            value_shape.translate(
                egui::Align2::CENTER_CENTER
                    .align_size_within_rect(
                        value_shape.visual_bounding_rect().size(),
                        ui.painter().clip_rect(),
                    )
                    .min
                    .to_vec2(),
            );
            ui.painter().add(value_shape);
            if ui.button("LaTeX").clicked() {
                let latex = value.to_latex();
                clipboard
                    .set_contents(latex)
                    .expect("Failed to copy LaTeX to clipboard!");
            }
            if ui.button("Echelon").clicked() {
                let echelon = match value {
                    Type::Scalar(_) => "1".to_string(),
                    Type::Matrix(m) => match m.echelon() {
                        Ok(Aftermath { result: _, steps }) => steps.join("\n"),
                        Err(err) => err.to_string(),
                    },
                };
                clipboard
                    .set_contents(echelon)
                    .expect("Failed to copy LaTeX to clipboard!");
            }
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
    }: &mut State<K>,
    locale: &Locale,
) {
    let mut run_shell_command = |shell_text: &mut String| match parse_instruction(shell_text, env) {
        Ok(identifier) => {
            shell_text.clear();
            windows.insert(identifier, WindowState { is_open: true });
        }
        Err(error) => {
            println!("{}", error);
            toasts.error(error.to_string(), Duration::from_secs(5));
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
