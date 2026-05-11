use crate::config::MatchField;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub index: usize,
    pub label: String,
    pub raw_text: String,
    pub icon_name: Option<String>,
    pub icon: Option<Arc<IconBitmap>>,
    pub exec: Option<String>,
    #[allow(dead_code)]
    pub path: Option<String>,
    pub terminal: bool,
    pub working_dir: Option<String>,
    pub search: SearchData,
    pub source: EntrySource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntrySource {
    Desktop,
    Dmenu,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub entry: Entry,
    pub score: i64,
    pub matched_indices: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchData {
    pub filename: String,
    pub name: String,
    pub generic: String,
    pub exec: String,
    pub keywords: String,
    pub categories: String,
    pub comment: String,
    pub combined: String,
}

impl SearchData {
    pub fn field(&self, field: MatchField) -> &str {
        match field {
            MatchField::Filename => &self.filename,
            MatchField::Name => &self.name,
            MatchField::Generic => &self.generic,
            MatchField::Exec => &self.exec,
            MatchField::Keywords => &self.keywords,
            MatchField::Categories => &self.categories,
            MatchField::Comment => &self.comment,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IconBitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
