use std::collections::HashMap;
use std::str::FromStr;
use egui::{Context, Sense, Ui};
use crate::environment::{Environment, Identifier, Type};
use crate::locale::Locale;
use crate::{State, WindowState};
use crate::env_gui::insert_to_env;
use crate::matrices::Matrix;
use crate::traits::MatrixNumber;

pub enum EditorType<K> {
    Matrix(usize, usize, Vec<K>, String),
    Scalar(K, String),
}

#[derive(Default)]
pub struct EditorState<K> {
    editor_type: Option<EditorType<K>>,
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

pub fn set_editor_to_matrix<K>(state: &mut EditorState<K>) where K: MatrixNumber {
    const DEFAULT_ROWS: usize = 2;
    const DEFAULT_COLS: usize = 3;
    state.editor_type = Some(EditorType::Matrix(
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

pub fn set_editor_to_scalar<K>(state: &mut EditorState<K>) where K: MatrixNumber {
    state.editor_type = Some(EditorType::Scalar(K::zero(), "".to_string()));
}

fn display_matrix_editor<K>(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (h, w, data, name): (&mut usize, &mut usize, &mut Vec<K>, &mut String),
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
        data.resize(*h * *w, K::zero());
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

fn display_k_editor<K>((i, j): (usize, usize), data: &mut Vec<K>, ui: &mut Ui, width: usize) -> bool where K: MatrixNumber + ToString {
    let id = i * width + j;
    let mut value = data[id].to_string();
    ui.add(egui::TextEdit::singleline(&mut value));
    match K::from_str(value.as_str()) {
        Ok(parsed) => {
            data[id] = parsed;
            true
        }
        Err(_) => false,
    }
}

// TODO: refactor this function with `display_matrix_editor`
fn display_scalar_editor<K>(
    ui: &mut Ui,
    env: &mut Environment<K>,
    windows: &mut HashMap<Identifier, WindowState>,
    (v, name): (&mut K, &mut String),
    locale: &Locale,
) -> bool where K: MatrixNumber + ToString {
    ui.label("Identifier:");
    ui.text_edit_singleline(name);
    ui.label(locale.get_translated("Enter value in the following format:"));


    let mut str_value = v.to_string();
    ui.horizontal(|ui| {
        ui.label("Value:");
        ui.add(egui::TextEdit::singleline(&mut str_value));
    });

    let new_value = K::from_str(str_value.as_str());

    if let Ok(new_value) = new_value {
        *v = new_value;
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
                    let value = Type::Scalar(v.clone());
                    insert_to_env(env, identifier, value, windows);
                    handled = true;
                }
                Err(_) => handled = false,
            }
        }
    });
    handled
}

