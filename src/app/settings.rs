#[derive(Clone, PartialEq)]
pub enum Column {
    Index(usize),
    Title(String),
    None,
}

#[derive(Debug, Clone)]
pub enum SortingMode {
    ValuesDesc,
    TitlesNumericAsc,
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
    pub input_file: Option<String>,
    pub x: Column,
    pub scales: Option<String>,
    pub paired: bool,
    pub sort_mode: SortingMode,
}
