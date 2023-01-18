use std::collections::HashMap;
use std::str::FromStr;
use egui::{Context, Sense, Ui};
use crate::environment::{Environment, Identifier, Type};
use crate::locale::Locale;
use crate::{State, WindowState};
use crate::env_gui::insert_to_env;
use crate::matrices::Matrix;
use crate::traits::MatrixNumber;

type DisplayType = String;

pub enum EditorType {
    Matrix(usize, usize, Vec<DisplayType>, String),
    Scalar(DisplayType, String),
}

#[derive(Default)]
pub struct EditorState {
    editor_type: Option<EditorType>,
}

pub fn display_editor<K>(ctx: &Context, state: &mut State, locale: &Locale) where K: MatrixNumber + ToString {
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
                        locale,
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

pub fn set_editor_to_matrix<K>(state: &mut EditorState) where K: MatrixNumber {
    const DEFAULT_ROWS: usize = 2;
    const DEFAULT_COLS: usize = 3;
    state.editor_type = Some(EditorType::Matrix(
        DEFAULT_ROWS,
        DEFAULT_COLS,
        vec![String::new(); DEFAULT_ROWS * DEFAULT_COLS],
        "".to_string(),
    ));
}

pub fn set_editor_to_scalar<K>(state: &mut EditorState) where K: MatrixNumber {
    state.editor_type = Some(EditorType::Scalar(String::new(), "".to_string()));
}

fn display_matrix_editor<K>(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (h, w, data, name): (&mut usize, &mut usize, &mut Vec<DisplayType>, &mut String),
    locale: &Locale,
) -> bool where K: MatrixNumber + ToString {
    ui.label("Identifier:");
    ui.text_edit_singleline(name);
    ui.label(locale.get_translated("Enter matrix in the following format:"));
    ui.label("Height:");
    ui.add(egui::DragValue::new(h));
    ui.label("Width:");
    ui.add(egui::DragValue::new(w));
    if data.len() != *h * *w {
        data.resize(*h * *w, String::new());
    }

    egui::Grid::new("matrix_editor").show(ui, |ui| {
        for i in 0..*h {
            for j in 0..*w {
                ui.label(format!("({}, {})", i, j));
                display_k_editor::<K>((i, j), data, ui, *w);
            }
            ui.end_row();
        }
    });

    let parsed_result = parse_all_data(&data);
    let is_identifier_valid = !name.is_empty() && Identifier::is_valid(name);
    let can_save = is_identifier_valid && parsed_result.is_some();
    let button_sense = if can_save {
        Sense::click()
    } else {
        Sense::hover()
    };
    let mut handled = false;
    ui.horizontal(|ui| {
        let add_button = ui.add(egui::Button::new(locale.get_translated("Add")).sense(button_sense));
        if !can_save && !is_identifier_valid {
            ui.label(locale.get_translated("Identifier is invalid!"));
        } else if !can_save && parsed_result.is_none() {
            ui.label(locale.get_translated("Matrix is invalid!"));
        }

        if add_button.clicked() {
            if let Some(parsed) = parsed_result {
                match Identifier::new(name.clone()) {
                    Ok(identifier) => {
                        let value = Type::Matrix(Matrix::from_vec(parsed, (*h, *w)).unwrap());
                        insert_to_env(env, identifier, value, windows);
                        handled = true;
                    }
                    Err(_) => handled = false,
                }
            } else {
                handled = false;
            }
        }
    });
    handled
}

fn parse_all_data<K>(data: &Vec<DisplayType>) -> Option<Vec<K>> where K: MatrixNumber {
    let mut result: Vec<K> = vec![];
    for e in data.iter() {
        result.push(K::from_str(&*e).ok()?)
    }
    Some(result)
}

fn display_k_editor<K>((i, j): (usize, usize), data: &mut Vec<DisplayType>, ui: &mut Ui, width: usize) -> bool where K: MatrixNumber + ToString {
    let id = i * width + j;
    ui.add(egui::TextEdit::singleline(&mut data[id]));
    true
}

// TODO: refactor this function with `display_matrix_editor`
fn display_scalar_editor<K>(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (v, name): (&mut DisplayType, &mut String),
    locale: &Locale,
) -> bool where K: MatrixNumber + ToString {
    ui.label("Identifier:");
    ui.text_edit_singleline(name);
    ui.label(locale.get_translated("Enter value in the following format:"));

    ui.horizontal(|ui| {
        ui.label("Value:");
        ui.add(egui::TextEdit::singleline(v));
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
            let parsed = K::from_str(v);
            if let Ok(parsed) = parsed {
                match Identifier::new(name.clone()) {
                    Ok(identifier) => {
                        let value = Type::Scalar(parsed);
                        insert_to_env(env, identifier, value, windows);
                        handled = true;
                    }
                    Err(_) => handled = false,
                }
            } else {
                handled = false
            }
        }
    });
    handled
}

