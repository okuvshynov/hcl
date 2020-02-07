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
    Batch, // Read file until EOF, send an update after every empty line. New data forms new 'DataSet'
    Autorefresh(Duration), // Read file until EOF, replace all at once, replace whole data. Repeat.
}

pub struct Settings {
    pub cmd: Option<Vec<String>>,
    pub refresh_rate: Duration,
    pub x: Column,
    pub epoch: Column,
    pub scales: Option<String>,
}

impl Settings {
    pub fn fetch_mode(&self) -> FetchMode {
        match (self.refresh_rate.as_nanos() > 0, self.epoch != Column::None) {
            (true, _) => FetchMode::Autorefresh(self.refresh_rate),
            (_, true) => FetchMode::Batch,
            _ => FetchMode::Incremental,
        }
    }
}
