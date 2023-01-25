mod constants;
mod editor_gui;
mod env_gui;
mod environment;
mod fractal_clock;
#[cfg(feature = "easter-eggs")]
mod furier;
mod locale;
mod matrices;
mod matrix_algorithms;
mod parser;
mod rationals;
mod traits;

use crate::constants::{APP_NAME, DEFAULT_HEIGHT, DEFAULT_LEFT_PANEL_WIDTH, DEFAULT_WIDTH};
use crate::editor_gui::{
    display_editor, set_editor_to_existing_matrix, set_editor_to_existing_scalar,
    set_editor_to_matrix, set_editor_to_scalar, EditorState,
};
use crate::environment::{Environment, Identifier, Type};
use crate::locale::{Language, Locale};
use crate::matrix_algorithms::Aftermath;
use crate::parser::parse_instruction;
use crate::traits::{GuiDisplayable, LaTeXable, MatrixNumber};
use clipboard::{ClipboardContext, ClipboardProvider};
use constants::{FONT_ID, TEXT_COLOR, VALUE_PADDING};
use eframe::{egui, IconData};

use egui::{vec2, Context, Response, Sense, Ui};
use env_gui::insert_to_env;
use num_rational::Rational64;
use std::collections::HashMap;
use std::default::Default;
use std::time::Duration;

use crate::fractal_clock::FractalClock;
#[cfg(feature = "easter-eggs")]
use crate::furier::Fourier;
use clap::builder::TypedValueParser;
use clap::Parser;

/// Field for matrices.
type F = Rational64;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
        icon_data: load_icon("assets\\icon.png"),
        ..Default::default()
    };
    let args = MatrixAppArgs::parse();
    let locale = Locale::new(args.language);
    eframe::run_native(
        &locale.get_translated(APP_NAME),
        options,
        Box::new(|_cc| Box::<MatrixApp<F>>::new(MatrixApp::new(locale))),
    )
}

fn load_icon(path: &str) -> Option<IconData> {
    let image = image::open(path).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "**Just Pure 2D Graphics Matrix Display** is a powerful matrix calculator written in Rust using egui."
)]
struct MatrixAppArgs {
    #[arg(
    long,
    default_value_t = Language::English,
    value_parser = clap::builder::PossibleValuesParser::new(["English", "Polish", "Spanish"])
    .map(|s| Language::of(Some(s))),
    )]
    language: Language,
}

pub struct WindowState {
    is_open: bool,
}

#[derive(Default)]
struct ShellState {
    text: String,
}

pub struct State<K: MatrixNumber> {
    env: Environment<K>,
    windows: HashMap<Identifier, WindowState>,
    shell: ShellState,
    editor: EditorState,
    toasts: egui_toast::Toasts,
    clipboard: ClipboardContext,
    clock: FractalClock,
    #[cfg(feature = "easter-eggs")]
    furier: Option<Fourier>,
}

impl<K: MatrixNumber> Default for State<K> {
    fn default() -> Self {
        Self {
            env: Default::default(),
            windows: Default::default(),
            shell: Default::default(),
            editor: Default::default(),
            toasts: Default::default(),
            clock: Default::default(),
            clipboard: ClipboardContext::new().expect("Failed to create Clipboard context!"),
            #[cfg(feature = "easter-eggs")]
            furier: Fourier::from_json_file("assets/dft_andrzej.json".to_string()).ok(),
        }
    }
}

struct MatrixApp<K: MatrixNumber> {
    state: State<K>,
    locale: Locale,
}

impl<K: MatrixNumber> MatrixApp<K> {
    fn new(locale: Locale) -> Self {
        Self {
            state: State::default(),
            locale,
        }
    }

    // Get Translated
    fn gt(&self, str: &str) -> String {
        self.locale.get_translated(str)
    }
}

impl<K: MatrixNumber> eframe::App for MatrixApp<K> {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let window_size = frame.info().window_info.size;
        self.state.toasts = egui_toast::Toasts::default()
            .anchor((window_size.x - 10., window_size.y - 40.))
            .direction(egui::Direction::BottomUp)
            .align_to_end(true);

        let _top_menu = display_menu_bar(ctx, &mut self.state, &self.locale);
        display_editor::<K>(ctx, &mut self.state, &self.locale);

        let _left_panel = egui::SidePanel::left("objects")
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
            })
            .response;

        let mut windows_result = None;
        for (id, window) in self.state.windows.iter_mut() {
            if window.is_open {
                let element = self.state.env.get(id).unwrap();
                let local_result = display_env_element_window(
                    ctx,
                    (id, element),
                    &self.locale,
                    &mut self.state.clipboard,
                    &mut self.state.editor,
                    &mut window.is_open,
                );
                windows_result = windows_result.or(local_result);
            }
        }

        if let Some(value) = windows_result {
            insert_to_env(
                &mut self.state.env,
                Identifier::result(),
                value,
                &mut self.state.windows,
            );
        }

        display_shell::<K>(ctx, &mut self.state, &self.locale);

        // Center panel has to be added last, otherwise the side panel will be on top of it.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.gt(APP_NAME));
            #[cfg(feature = "easter-eggs")]
            match &mut self.state.furier {
                Some(furier) => {
                    furier.ui(ui, _left_panel.rect.width(), _top_menu.rect.height());
                }
                None => {
                    self.state.clock.ui(ui, Some(seconds_since_midnight()));
                }
            }
            #[cfg(not(feature = "easter-eggs"))]
            self.state.clock.ui(ui, Some(seconds_since_midnight()));
        });

        self.state.toasts.show(ctx);
    }
}

fn seconds_since_midnight() -> f64 {
    use chrono::Timelike;
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}

fn display_menu_bar<K: MatrixNumber>(
    ctx: &Context,
    state: &mut State<K>,
    locale: &Locale,
) -> Response {
    egui::TopBottomPanel::top("menu_bar")
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                display_add_matrix_button(ui, state, locale);
                display_add_scalar_button(ui, state, locale);
            })
        })
        .response
}

fn display_add_matrix_button<K: MatrixNumber>(ui: &mut Ui, state: &mut State<K>, locale: &Locale) {
    if ui.button(locale.get_translated("Add Matrix")).clicked() {
        set_editor_to_matrix(&mut state.editor, &K::zero().to_string());
    }
}

fn display_add_scalar_button<K: MatrixNumber>(ui: &mut Ui, state: &mut State<K>, locale: &Locale) {
    if ui.button(locale.get_translated("Add Scalar")).clicked() {
        set_editor_to_scalar(&mut state.editor, &K::zero().to_string());
    }
}

fn display_env_element<K: MatrixNumber>(
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

fn display_env_element_window<K: MatrixNumber>(
    ctx: &Context,
    (identifier, value): (&Identifier, &Type<K>),
    locale: &Locale,
    clipboard: &mut ClipboardContext,
    editor: &mut EditorState,
    is_open: &mut bool,
) -> Option<Type<K>> {
    let mut window_result = None;

    egui::Window::new(identifier.to_string())
        .open(is_open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("LaTeX").clicked() {
                    let latex = value.to_latex();
                    clipboard
                        .set_contents(latex)
                        .expect("Failed to copy LaTeX to clipboard!");
                }
                if let Type::Matrix(m) = value {
                    if ui.button(locale.get_translated("Echelon")).clicked() {
                        let echelon = match m.echelon() {
                            Ok(Aftermath { result, steps }) => {
                                window_result = Some(Type::Matrix(result));
                                steps.join("\n")
                            }
                            Err(err) => err.to_string(),
                        };
                        clipboard
                            .set_contents(echelon)
                            .expect("Failed to copy LaTeX to clipboard!");
                    }
                }
                if ui.button(locale.get_translated("Inverse")).clicked() {
                    let inverse = match value {
                        Type::Scalar(s) => match K::one().checked_div(s) {
                            Some(inv) => {
                                window_result = Some(Type::Scalar(inv.clone()));
                                inv.to_latex()
                            }
                            None => "Failed to calculate inverse".to_string(),
                        },
                        Type::Matrix(m) => match m.inverse() {
                            Ok(Aftermath { result, steps }) => {
                                window_result = Some(Type::Matrix(result));
                                steps.join("\n")
                            }
                            Err(err) => err.to_string(),
                        },
                    };
                    clipboard
                        .set_contents(inverse)
                        .expect("Failed to copy LaTeX to clipboard!");
                }
            });
            let mut value_shape = value.to_shape(ctx, FONT_ID, TEXT_COLOR);
            let value_rect = value_shape.visual_bounding_rect();

            ui.set_min_width(value_rect.width() + 2. * VALUE_PADDING);
            ui.set_max_width(ui.min_size().x);
            ui.separator();

            let bar_height = ui.min_size().y;

            ui.add_space(value_rect.height() + VALUE_PADDING);

            value_shape.translate(
                ui.clip_rect().min.to_vec2()
                    + vec2(
                        (ui.min_size().x - value_rect.width()) / 2.,
                        bar_height + VALUE_PADDING,
                    ),
            );
            ui.painter().add(value_shape);

            if !identifier.is_result() {
                ui.separator();
                if ui.button(locale.get_translated("Edit")).clicked() {
                    match value {
                        Type::Scalar(s) => {
                            set_editor_to_existing_scalar(editor, s, identifier.to_string())
                        }
                        Type::Matrix(m) => {
                            set_editor_to_existing_matrix(editor, m, identifier.to_string())
                        }
                    }
                }
            };
        });

    window_result
}

fn display_shell<K: MatrixNumber>(
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
