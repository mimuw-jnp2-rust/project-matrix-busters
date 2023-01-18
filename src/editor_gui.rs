use crate::env_gui::insert_to_env;
use crate::environment::{Environment, Identifier, Type};
use crate::locale::Locale;
use crate::matrices::Matrix;
use crate::traits::MatrixNumber;
use crate::{State, WindowState};
use anyhow::Context;
use egui::{Sense, Ui};
use std::collections::HashMap;

pub enum EditorType {
    Matrix(usize, usize, Vec<String>),
    Scalar(String),
}

pub struct EditorContent {
    identifier_name: String,
    editor_type: EditorType,
}

#[derive(Default)]
pub struct EditorState {
    editor_content: Option<EditorContent>,
}

pub fn display_editor<K>(ctx: &egui::Context, state: &mut State<K>, locale: &Locale)
where
    K: MatrixNumber + ToString,
{
    if let Some(editor_content) = &mut state.editor.editor_content {
        let result = display_editor_is_some::<K>(
            ctx,
            editor_content,
            &mut state.env,
            &mut state.windows,
            locale,
        );
        match result {
            Ok(true) => {
                state.editor.editor_content = None;
            }
            Ok(false) => {}
            Err(_) => {
                panic!("It shouldn't return Err")
            }
        }
    }
}

pub fn set_editor_to_matrix(state: &mut EditorState, def: &str) {
    const DEFAULT_ROWS: usize = 2;
    const DEFAULT_COLS: usize = 3;
    state.editor_content = Some(EditorContent {
        identifier_name: "".to_string(),
        editor_type: EditorType::Matrix(
            DEFAULT_ROWS,
            DEFAULT_COLS,
            vec![String::from(def); DEFAULT_ROWS * DEFAULT_COLS],
        ),
    });
}

pub fn set_editor_to_scalar(state: &mut EditorState, def: &str) {
    state.editor_content = Some(EditorContent {
        identifier_name: "".to_string(),
        editor_type: EditorType::Scalar(String::from(def)),
    });
}

fn display_editor_is_some<K>(
    ctx: &egui::Context,
    content: &mut EditorContent,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    locale: &Locale,
) -> anyhow::Result<bool>
where
    K: MatrixNumber,
{
    let mut handled: anyhow::Result<bool> = Ok(false);
    let mut editor_opened = true;

    egui::Window::new(locale.get_translated("Editor"))
        .open(&mut editor_opened)
        .show(ctx, |ui| {
            let EditorContent {
                identifier_name,
                editor_type,
            } = content;
            ui.label("Identifier:");
            ui.text_edit_singleline(identifier_name);
            let result = match editor_type {
                EditorType::Matrix(h, w, data) => {
                    display_matrix_editor((h, w), data, ui, locale);
                    parse_matrix_data::<K>((h, w), data)
                }
                EditorType::Scalar(data) => {
                    display_scalar_editor(data, ui, locale);
                    parse_scalar_data::<K>(data)
                }
            };
            let mut err_msg = if Identifier::is_valid(identifier_name) {
                None
            } else {
                Some(String::from("Identifier is invalid!"))
            };
            if let Err(err) = &result {
                err_msg = Some(format!("Matrix is invalid! {err}"));
            };
            ui.horizontal(|ui| {
                let sense = if err_msg.is_some() {
                    Sense::hover()
                } else {
                    Sense::click()
                };
                let add_button =
                    ui.add(egui::Button::new(locale.get_translated("Add")).sense(sense));
                if add_button.clicked() {
                    if let Some(some) = &err_msg {
                        ui.label(locale.get_translated("Error") + some);
                    } else {
                        insert_to_env(
                            env,
                            Identifier::new(identifier_name.to_string()).expect("Should work"),
                            result.expect("There should be a value."),
                            windows,
                        );
                        handled = Ok(true);
                    }
                };
            })
        });

    // Editor was closed
    if !editor_opened {
        handled = Ok(true);
    }

    handled
}

fn parse_matrix_data<K>(
    (h, w): (&mut usize, &mut usize),
    data: &mut [String],
) -> anyhow::Result<Type<K>>
where
    K: MatrixNumber,
{
    let mut result: Vec<K> = vec![];
    for e in data.iter() {
        result.push(K::from_str(e).ok().context("xd")?)
    }
    Ok(Type::Matrix(Matrix::from_vec(result, (*h, *w))?))
}

fn parse_scalar_data<K>(data: &mut str) -> anyhow::Result<Type<K>>
where
    K: MatrixNumber,
{
    Ok(Type::Scalar(
        K::from_str(data).ok().context("Invalid cast!")?,
    ))
}

fn display_matrix_editor(
    (h, w): (&mut usize, &mut usize),
    data: &mut Vec<String>,
    ui: &mut Ui,
    locale: &Locale,
) {
    ui.label(locale.get_translated("Enter the matrix:"));
    ui.horizontal(|ui| {
        ui.label("Height");
        ui.add(egui::DragValue::new(h));
    });
    ui.horizontal(|ui| {
        ui.label("Width");
        ui.add(egui::DragValue::new(w));
    });
    ui.separator();
    if data.len() != *h * *w {
        data.resize(*h * *w, String::from("0"));
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
}

fn display_k_editor((i, j): (usize, usize), data: &mut [String], ui: &mut Ui, width: usize) {
    let id = i * width + j;
    ui.add(egui::TextEdit::singleline(&mut data[id]));
}

fn display_scalar_editor(data: &mut String, ui: &mut Ui, locale: &Locale) {
    ui.label(locale.get_translated("Enter the scalar:"));
    ui.add(egui::TextEdit::singleline(data));
}
