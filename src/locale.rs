use lazy_static::lazy_static;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    Polish,
}

impl Language {
    pub fn of(str: Option<String>) -> Language {
        str.map_or(Language::English, |str| match str.as_str() {
            "en" | "english" => Language::English,
            "es" | "spanish" => Language::Spanish,
            "pl" | "polish" => Language::Polish,
            _ => Language::English,
        })
    }
}

#[allow(dead_code)]
pub struct Locale {
    language: Language,
    translation_map: HashMap<String, String>,
}

impl Locale {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            translation_map: gen_map(match language {
                Language::English => &TRANS_EN_RAW,
                Language::Polish => &TRANS_PL_RAW,
                Language::Spanish => &TRANS_ES_RAW,
            }),
        }
    }

    fn unwrap_or_default(str: Option<&String>, default: &str) -> String {
        str.unwrap_or(&default.to_string()).as_str().to_owned()
    }

    pub fn get_translated(&self, s: &str) -> String {
        Self::unwrap_or_default(self.translation_map.get(s), s)
    }

    #[allow(dead_code)]
    pub fn get_translated_from(&self, s: String) -> String {
        self.get_translated(&s)
    }
}

lazy_static! {
    pub static ref TRANS_EN_RAW: Vec<(&'static str, &'static str)> = vec![
        ("objects", "Objects"),
        ("matrix", "Matrix"),
        ("Add Matrix", "Add Matrix"),
        ("Add Scalar", "Add Scalar"),
        ("JP2GMD - Matrix Calculator", "JP2GMD - Matrix Calculator"),
        ("Echelon", "Echelon"),
        ("Run", "Run"),
        ("Editor", "Editor"),
        ("Identifier:", "Identifier:"),
        ("Matrix is invalid!", "Matrix is invalid!"),
        ("Add", "Add"),
        ("Error", "Error"),
        ("Enter the matrix:", "Enter the matrix:"),
        ("Enter the scalar:", "Enter the scalar:"),
    ];
    pub static ref TRANS_PL_RAW: Vec<(&'static str, &'static str)> = vec![
        ("objects", "Obiekty"),
        ("matrix", "Macierz"),
        ("Add Matrix", "Dodaj Macierz"),
        ("Add Scalar", "Dodaj Skalar"),
        ("JP2GMD - Matrix Calculator", "JP2GMD - Kalkulator Macierzy"),
        ("Echelon", "Schodkuj"),
        ("Run", "Uruchom"),
        ("Editor", "Edytor"),
        ("Identifier:", "Identyfikator:"),
        ("Matrix is invalid!", "Macierz jest niepoprawna!"),
        ("Add", "Dodaj"),
        ("Error", "Błąd"),
        ("Enter the matrix:", "Wprowadź macierz:"),
        ("Enter the scalar:", "Wprowadź skalar:"),
    ];
    pub static ref TRANS_ES_RAW: Vec<(&'static str, &'static str)> = vec![
        ("objects", "Objetos"),
        ("matrix", "Matriz"),
        ("Add Matrix", "Añadir Matriz"),
        ("Add Scalar", "Añadir Escalar"),
        (
            "JP2GMD - Matrix Calculator",
            "JP2GMD - Calculadora de Matrices"
        ),
        ("Echelon", "Echelon"),
        ("Run", "Ejecutar"),
        ("Editor", "Editor"),
        ("Identifier:", "Identificador:"),
        ("Matrix is invalid!", "¡La matriz es inválida!"),
        ("Add", "Añadir"),
        ("Error", "Error"),
        ("Enter the matrix:", "Introduzca la matriz:"),
        ("Enter the scalar:", "Introduzca el escalar:"),
    ];
}

fn gen_map(vec: &[(&'static str, &'static str)]) -> HashMap<String, String> {
    vec.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
