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

#[derive(Debug)]
pub enum FetchMode {
    Incremental, // Read file until EOF, append line by line, as soon as new data arrives.
    Autorefresh(Duration), // Read file until EOF, replace all at once, replace whole data. Repeat.
}

pub struct Settings {
    pub cmd: Option<Vec<String>>,
    pub refresh_rate: Duration,
    pub x: Column,
    pub scales: Option<String>,
}

impl Settings {
    pub fn fetch_mode(&self) -> FetchMode {
        if self.refresh_rate.as_nanos() > 0 {
            FetchMode::Autorefresh(self.refresh_rate)
        } else {
            FetchMode::Incremental
        }
    }
}
