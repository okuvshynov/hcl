use std::iter;

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
    pub x: Option<String>,
    pub y: Vec<f64>,
}

impl Slice {
    pub fn default() -> Slice {
        Slice { x: None, y: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct SeriesSet {
    pub x: Option<(String, Vec<String>)>,
    pub y: Vec<Series>,
}

impl SeriesSet {
    pub fn default() -> SeriesSet {
        SeriesSet { y: vec![], x: None }
    }

    pub fn series_size(&self) -> i64 {
        self.y.first().map(|s| s.values.len() as i64).unwrap_or(0)
    }

    pub fn series_count(&self) -> i64 {
        self.y.len() as i64
    }

    pub fn append_slice(&mut self, slice: Slice) {
        if let (Some((_, x)), Some(xn)) = (self.x.as_mut(), slice.x.as_ref()) {
            x.push(xn.to_owned());
        }
        // here we pad the slice with 0 (should be NaN?) if it's shorter
        self.y
            .iter_mut()
            .zip(slice.y.iter().chain(iter::repeat(&std::f64::NAN)))
            .for_each(|(y, v)| y.values.push(*v));
    }

    // TODO: other assumed to be empty
    pub fn append_set(&mut self, mut other: SeriesSet) {
        let old_length = self.series_size();
        let mut used = vec![false; self.series_count() as usize];

        other.y.iter_mut().for_each(|ns| {
            ns.values = if let Some((idx, old_series)) = self
                .y
                .iter_mut()
                .enumerate()
                .find(|(_, os)| os.title == ns.title)
            {
                used[idx] = true;
                let mut new_series = vec![];
                new_series.append(&mut old_series.values);
                new_series
            } else {
                vec![std::f64::NAN; old_length as usize]
            };
        });

        let mut i = 0;
        self.y.retain(|_| !(used[i], i += 1).0);

        other.y.append(&mut self.y);
        self.y = other.y;

        if let (Some((_, xo)), Some((_, xn))) = (self.x.as_mut(), other.x.as_mut()) {
            xo.append(xn);
        }
    }
}
