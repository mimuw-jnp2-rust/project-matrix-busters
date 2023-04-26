use crate::env_gui::insert_to_env;
use crate::environment::{Environment, Identifier, Type};
use crate::matrices::Matrix;
use crate::parser::parse_expression;
use crate::traits::MatrixNumber;
use crate::{State, WindowState};
use anyhow::bail;
use egui::{Sense, Ui};
use locale::Locale;
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

pub fn display_editor<K: MatrixNumber>(ctx: &egui::Context, state: &mut State<K>, locale: &Locale) {
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

pub fn set_editor_to_existing_matrix<K: MatrixNumber>(
    state: &mut EditorState,
    matrix: &Matrix<K>,
    name: String,
) {
    let (w, h) = matrix.get_shape();
    let data: Vec<K> = matrix.clone().into();
    let data = data.iter().map(|el| el.to_string()).collect();
    state.editor_content = Some(EditorContent {
        identifier_name: name,
        editor_type: EditorType::Matrix(w, h, data),
    });
}

pub fn set_editor_to_scalar(state: &mut EditorState, def: &str) {
    state.editor_content = Some(EditorContent {
        identifier_name: "".to_string(),
        editor_type: EditorType::Scalar(String::from(def)),
    });
}

pub fn set_editor_to_existing_scalar<K: MatrixNumber>(
    state: &mut EditorState,
    value: &K,
    name: String,
) {
    state.editor_content = Some(EditorContent {
        identifier_name: name,
        editor_type: EditorType::Scalar(value.to_string()),
    })
}

fn display_editor_is_some<K: MatrixNumber>(
    ctx: &egui::Context,
    content: &mut EditorContent,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    locale: &Locale,
) -> anyhow::Result<bool> {
    let mut handled: anyhow::Result<bool> = Ok(false);
    let mut editor_opened = true;

    egui::Window::new(locale.get_translated("Editor"))
        .open(&mut editor_opened)
        .show(ctx, |ui| {
            let EditorContent {
                identifier_name,
                editor_type,
            } = content;
            ui.label(locale.get_translated("Identifier:"));
            ui.text_edit_singleline(identifier_name);
            let result = match editor_type {
                EditorType::Matrix(h, w, data) => {
                    display_matrix_editor((h, w), data, ui, locale);
                    parse_matrix_data::<K>((h, w), data, env)
                }
                EditorType::Scalar(data) => {
                    display_scalar_editor(data, ui, locale);
                    parse_scalar_data::<K>(data, env)
                }
            };
            let mut err_msg = if Identifier::is_valid(identifier_name) {
                None
            } else {
                Some(locale.get_translated("Identifier is invalid!"))
            };
            if let Err(err) = &result {
                err_msg = Some(
                    locale.get_translated("Matrix is invalid!") + "\n" + err.to_string().as_str(),
                );
            };
            ui.horizontal(|ui| {
                let sense = if err_msg.is_some() {
                    Sense::hover()
                } else {
                    Sense::click()
                };
                let add_button =
                    ui.add(egui::Button::new(locale.get_translated("Add")).sense(sense));
                if let Some(some) = &err_msg {
                    ui.label(locale.get_translated("Error ") + some);
                } else if add_button.clicked() {
                    insert_to_env(
                        env,
                        Identifier::new(identifier_name.to_string()).expect("Should work"),
                        result.expect("There should be a value."),
                        windows,
                    );
                    handled = Ok(true);
                };
            })
        });

    // Editor was closed
    if !editor_opened {
        handled = Ok(true);
    }

    handled
}

fn parse_matrix_data<K: MatrixNumber>(
    (h, w): (&mut usize, &mut usize),
    data: &mut [String],
    env: &Environment<K>,
) -> anyhow::Result<Type<K>> {
    let mut result: Vec<K> = vec![];
    for element in data.iter() {
        result.push(parse_scalar_with_env(element, env)?)
    }
    Ok(Type::Matrix(Matrix::from_vec(result, (*h, *w))?))
}

fn parse_scalar_data<K: MatrixNumber>(
    data: &mut str,
    env: &Environment<K>,
) -> anyhow::Result<Type<K>> {
    Ok(Type::Scalar(parse_scalar_with_env(data, env)?))
}

fn parse_scalar_with_env<K: MatrixNumber>(data: &str, env: &Environment<K>) -> anyhow::Result<K> {
    match parse_expression(data, env)? {
        Type::Scalar(scalar) => Ok(scalar),
        Type::Matrix(_) => bail!("Invalid expression! Result is not a scalar."),
    }
}

fn display_matrix_editor(
    (h, w): (&mut usize, &mut usize),
    data: &mut Vec<String>,
    ui: &mut Ui,
    locale: &Locale,
) {
    ui.label(locale.get_translated("Enter the matrix:"));
    egui::Grid::new("dimensions").show(ui, |ui| {
        ui.label(locale.get_translated("Height"));
        ui.add(egui::DragValue::new(h));
        ui.end_row();
        ui.label(locale.get_translated("Width"));
        ui.add(egui::DragValue::new(w));
        ui.end_row();
    });
    ui.separator();
    if data.len() != *h * *w {
        data.resize(*h * *w, String::from("0"));
    }

    egui::Grid::new("matrix_editor").show(ui, |ui| {
        ui.label("");
        for j in 0..*w {
            ui.label(format!("{}", j + 1).as_str());
        }
        ui.end_row();
        for i in 0..*h {
            ui.label(format!("{}", i + 1).as_str());
            for j in 0..*w {
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
