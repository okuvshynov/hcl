use std::time::Duration;

#[derive(Clone, PartialEq)]
pub enum Column {
    Index(usize),
    Title(String),
    None,
}

impl Column {
    pub fn matches(&self, title: &str, index: usize) -> bool {
        match self {
            Column::None => false,
            Column::Index(i) => *i == index,
            Column::Title(t) => t == title,
        }
    }
}

pub struct Settings {
    pub cmd: Option<Vec<String>>,
    pub refresh_rate: Duration,
    pub x: Column,
    pub epoch: Column,
    pub scales: Option<String>,
}
