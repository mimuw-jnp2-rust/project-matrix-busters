use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    English,
    Spanish,
    Polish,
}

fn gen_map(vec: &[(&'static str, &'static str)]) -> HashMap<String, String> {
    vec.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

lazy_static! {
    pub static ref TRANS_EN_RAW: Vec<(&'static str, &'static str)> =
        vec![("objects", "Objects"), ("matrix", "Matrix"),];
    pub static ref TRANS_PL_RAW: Vec<(&'static str, &'static str)> =
        vec![("objects", "Obiekty"), ("matrix", "Macierz"),];
    pub static ref TRANS_ES_RAW: Vec<(&'static str, &'static str)> =
        vec![("objects", "Objetos"), ("matrix", "Matriz"),];
    pub static ref TRANSLATIONS_EN: HashMap<String, String> = gen_map(&TRANS_EN_RAW);
    pub static ref TRANSLATIONS_ES: HashMap<String, String> = gen_map(&TRANS_ES_RAW);
    pub static ref TRANSLATIONS_PL: HashMap<String, String> = gen_map(&TRANS_PL_RAW);
    pub static ref LANGUAGES_SETS: HashMap<Locale, &'static HashMap<String, String>> = {
        HashMap::from([
            (Locale::English, &*TRANSLATIONS_EN),
            (Locale::Polish, &*TRANSLATIONS_PL),
            (Locale::Spanish, &*TRANSLATIONS_ES),
        ])
    };
}

// TODO: Add a function to get the current locale from the system.
pub static LANG: Locale = Locale::English;

#[allow(dead_code)]
pub fn get_translated_from(s: String) -> String {
    LANGUAGES_SETS
        .get(&LANG)
        .unwrap()
        .get(&s)
        .unwrap_or(&s.to_string())
        .as_str()
        .to_owned()
}

pub fn get_translated(s: &str) -> String {
    LANGUAGES_SETS
        .get(&LANG)
        .unwrap()
        .get(s)
        .unwrap_or(&s.to_string())
        .as_str()
        .to_owned()
}
