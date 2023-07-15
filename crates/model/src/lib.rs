use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
use sea_orm::{self, DeriveActiveEnum};

use strum;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NestedDictionaryItem {
    pub vocabulary_id: i32,
    pub target: String,
    pub target_lang: Language,
    pub lan_dict: HashMap<Language, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(target_family = "wasm", derive(strum::EnumIter, ))]
#[cfg_attr(
    not(target_family = "wasm"),
    derive(sea_orm::EnumIter, DeriveActiveEnum),
    sea_orm(rs_type = "String", db_type = "String(Some(3))")
)]
pub enum Language {
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "cht"))]
    Cht,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "cn"))]
    Cn,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "de"))]
    De,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "en"))]
    En,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "es"))]
    Es,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "fr"))]
    Fr,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "id"))]
    Id,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "jp"))]
    Jp,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "kr"))]
    Kr,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "pt"))]
    Pt,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "ru"))]
    Ru,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "th"))]
    Th,
    #[cfg_attr(not(target_family = "wasm"), sea_orm(string_value = "vi"))]
    Vi,
}

impl Language {
    pub fn str_id(&self) -> &str {
        use Language::*;
        match self {
            Cht => "cht",
            Cn => "cn",
            De => "de",
            En => "en",
            Es => "es",
            Fr => "fr",
            Id => "id",
            Jp => "jp",
            Kr => "kr",
            Pt => "pt",
            Ru => "ru",
            Th => "th",
            Vi => "vi",
        }
    }
}

#[cfg(target_family = "wasm")]
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str_id())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SearchParams {
    pub search_word: String,
    pub batch_size: u64,
    pub page: Option<u64>,
}