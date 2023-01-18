use lazy_static::lazy_static;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    Polish,
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
        ("Add Matrix", "Add Matrix")
    ];
    pub static ref TRANS_PL_RAW: Vec<(&'static str, &'static str)> = vec![
        ("objects", "Obiekty"),
        ("matrix", "Macierz"),
        ("Add Matrix", "Dodaj Macierz")
    ];
    pub static ref TRANS_ES_RAW: Vec<(&'static str, &'static str)> = vec![
        ("objects", "Objetos"),
        ("matrix", "Matriz"),
        ("Add Matrix", "Añadir Matriz")
    ];
}

fn gen_map(vec: &[(&'static str, &'static str)]) -> HashMap<String, String> {
    vec.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
