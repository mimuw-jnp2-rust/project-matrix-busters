use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    Polish,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Spanish => write!(f, "Spanish"),
            Language::Polish => write!(f, "Polish"),
        }
    }
}

impl Language {
    pub fn of(str: Option<String>) -> Language {
        str.map_or(Language::English, |str| match str.to_lowercase().as_str() {
            "en" | "english" => Language::English,
            "es" | "spanish" => Language::Spanish,
            "pl" | "polish" => Language::Polish,
            _ => Language::English,
        })
    }
}

pub struct Locale {
    language: Language,
    translation_map: &'static HashMap<String, String>,
}

impl Locale {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            translation_map: match language {
                Language::English => &TRANS_EN_MAP,
                Language::Polish => &TRANS_PL_MAP,
                Language::Spanish => &TRANS_ES_MAP,
            },
        }
    }

    fn unwrap_or_default(str: Option<&String>, default: &str) -> String {
        match str {
            Some(str) => str.to_string(),
            None => default.to_string(),
        }
    }

    pub fn get_language(&self) -> Language {
        self.language
    }

    pub fn get_translated(&self, s: &str) -> String {
        Self::unwrap_or_default(self.translation_map.get(s), s)
    }

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
        ("Inverse", "Inverse"),
        ("Transpose", "Transpose"),
        ("Run", "Run"),
        ("Editor", "Editor"),
        ("Identifier:", "Identifier:"),
        ("Matrix is invalid!", "Matrix is invalid!"),
        ("Add", "Add"),
        ("Error", "Error"),
        ("Enter the matrix:", "Enter the matrix:"),
        ("Enter the scalar:", "Enter the scalar:"),
        ("Height", "Height"),
        ("Width", "Width"),
        ("Edit", "Edit"),
        ("Error ", "Error "),
        ("Identifier is invalid!", "Identifier is invalid!"),
        ("LaTeX copied to clipboard", "LaTeX copied to clipboard"),
        ("Failed to generate LaTeX", "Failed to generate LaTeX"),
        ("Language", "Language"),
        ("English", "English"),
        ("Polish", "Polish"),
        ("Spanish", "Spanish"),
    ];
    pub static ref TRANS_PL_RAW: Vec<(&'static str, &'static str)> =
        vec![
        ("objects", "Obiekty"),
        ("matrix", "Macierz"),
        ("Add Matrix", "Dodaj Macierz"),
        ("Add Scalar", "Dodaj Skalar"),
        (
            "JP2GMD - Matrix Calculator",
            "Jaki Potężny 2-wymiarowy Generator Macierzy Diagonalizowalnych - Kalkulator Macierzy"
        ),
        ("Echelon", "Schodkuj"),
        ("Inverse", "Odwrotność"),
        ("Transpose", "Transponuj"),
        ("Run", "Uruchom"),
        ("Editor", "Edytor"),
        ("Identifier:", "Identyfikator:"),
        ("Matrix is invalid!", "Macierz jest niepoprawna!"),
        ("Add", "Dodaj"),
        ("Error", "Błąd"),
        ("Enter the matrix:", "Wprowadź macierz:"),
        ("Enter the scalar:", "Wprowadź skalar:"),
        ("Height", "Wysokość"),
        ("Width", "Szerokość"),
        ("Edit", "Edytuj"),
        ("Error ", "Błąd "),
        ("Identifier is invalid!", "Identyfikator jest niepoprawny!"),
        ("LaTeX copied to clipboard", "Skopiowano LaTeX'a do schowka"),
        ("Failed to generate LaTeX", "Nie udało się wygenerować LaTeX'a"),
        ("Language", "Język"),
        ("English", "Angielski"),
        ("Polish", "Polski"),
        ("Spanish", "Hiszpański"),
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
        ("Inverse", "Inversa"),
        ("Transpose", "Transpuesta"),
        ("Run", "Ejecutar"),
        ("Editor", "Editor"),
        ("Identifier:", "Identificador:"),
        ("Matrix is invalid!", "¡La matriz es inválida!"),
        ("Add", "Añadir"),
        ("Error", "Error"),
        ("Enter the matrix:", "Introduzca la matriz:"),
        ("Enter the scalar:", "Introduzca el escalar:"),
        ("Height", "Altura"),
        ("Width", "Anchura"),
        ("Editor", "Editar"),
        ("Error ", "Error "),
        ("Identifier is invalid!", "¡El identificador es inválido!"),
        ("LaTeX copied to clipboard", "LaTeX copiado al portapapeles"),
        ("Failed to generate LaTeX", "No se pudo generar LaTeX"),
        ("Language", "Idioma"),
        ("English", "Inglés"),
        ("Polish", "Polaco"),
        ("Spanish", "Español"),
    ];
    pub static ref TRANS_EN_MAP: HashMap<String, String> = gen_map(&TRANS_EN_RAW);
    pub static ref TRANS_PL_MAP: HashMap<String, String> = gen_map(&TRANS_PL_RAW);
    pub static ref TRANS_ES_MAP: HashMap<String, String> = gen_map(&TRANS_ES_RAW);
}

fn gen_map(vec: &[(&'static str, &'static str)]) -> HashMap<String, String> {
    vec.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_three_languages() {
        assert_eq!(Language::of(Some(String::from("en"))), Language::English);
        assert_eq!(Language::of(Some(String::from("pl"))), Language::Polish);
        assert_eq!(Language::of(Some(String::from("es"))), Language::Spanish);
    }

    #[test]
    fn test_three_long_languages() {
        assert_eq!(
            Language::of(Some(String::from("english"))),
            Language::English
        );
        assert_eq!(Language::of(Some(String::from("polish"))), Language::Polish);
        assert_eq!(
            Language::of(Some(String::from("spanish"))),
            Language::Spanish
        );
    }
}
