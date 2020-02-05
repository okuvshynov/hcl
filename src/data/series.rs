#[derive(Debug, Clone)]
pub struct Series {
    pub title: String,
    pub values: Vec<f64>,
}

impl Series {
    pub fn with_title(title: &str) -> Series {
        Series {
            title: title.to_string(),
            values: vec![],
        }
    }
}

pub struct Slice {
    pub epoch: Option<String>,
    pub x: Option<String>,
    pub y: Vec<f64>,
}

impl Slice {
    pub fn default() -> Slice {
        Slice {
            epoch: None,
            x: None,
            y: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeriesSet {
    pub epoch: Option<String>,
    pub x: Option<(String, Vec<String>)>,
    pub y: Vec<Series>,
}

impl SeriesSet {
    pub fn default() -> SeriesSet {
        SeriesSet {
            y: vec![],
            x: None,
            epoch: None,
        }
    }

    pub fn series_size(&self) -> i64 {
        self.y.first().map(|s| s.values.len() as i64).unwrap_or(0)
    }

    pub fn series_count(&self) -> i64 {
        self.y.len() as i64
    }

    pub fn append_slice(&mut self, slice: Slice) {
        if self.y.len() != slice.y.len() {
            // TODO: log warning
            return;
        }
        self.epoch = slice.epoch;
        if let (Some((_, x)), Some(xn)) = (self.x.as_mut(), slice.x.as_ref()) {
            x.push(xn.to_owned());
        }
        self.y
            .iter_mut()
            .zip(slice.y.iter())
            .for_each(|(y, v)| y.values.push(*v));
    }
}
