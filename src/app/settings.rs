use std::time::Duration;

#[derive(Clone)]
pub enum XColumn {
    Index(usize),
    Title(String),
    None,
}

pub struct Settings {
    pub cmd: Option<Vec<String>>,
    pub refresh_rate: Duration,
    pub x: XColumn,
    pub scales: Option<String>,
}
