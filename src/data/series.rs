use std::f64::NAN;
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
            .zip(slice.y.iter().chain(iter::repeat(&NAN)))
            .for_each(|(y, v)| y.values.push(*v));
    }

    pub fn append_set(&mut self, mut other: SeriesSet) {
        let old_length = self.series_size();
        let new_length = other.series_size();
        let mut used = vec![false; self.series_count() as usize];

        other.y.iter_mut().for_each(|ns| {
            let mut all_values = if let Some((idx, old_series)) = self
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
                vec![NAN; old_length as usize]
            };
            all_values.append(&mut ns.values);
            ns.values = all_values;
        });

        let mut i = 0;
        self.y.retain(|_| !(used[i], i += 1).0);

        self.y.iter_mut().for_each(|os| {
            os.values.append(&mut vec![NAN; new_length as usize]);
        });

        other.y.append(&mut self.y);
        self.y = other.y;

        if let (Some((_, xo)), Some((_, xn))) = (self.x.as_mut(), other.x.as_mut()) {
            xo.append(xn);
        }
        self.order_by();
    }

    fn order_by(&mut self) {
        self.y.sort_by_cached_key(|a| - (a.values.iter().filter(|v| !v.is_nan()).sum::<f64>() * 1.0e9) as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_set() {
        let mut old = SeriesSet {
            x: None,
            y: vec![
                Series {
                    title: "a".to_owned(),
                    values: vec![1.0, 2.0, 3.0],
                },
                Series {
                    title: "b".to_owned(),
                    values: vec![2.0, 3.0, 4.0],
                },
            ],
        };

        let new = SeriesSet {
            x: None,
            y: vec![
                Series {
                    title: "a".to_owned(),
                    values: vec![4.0, 5.0],
                },
                Series {
                    title: "c".to_owned(),
                    values: vec![6.0, 7.0],
                },
            ],
        };

        old.append_set(new);

        assert_eq!(old.y.len(), 3);
        assert_eq!(old.y[0].title, "a".to_owned());
        assert_eq!(old.y[1].title, "c".to_owned());
        assert_eq!(old.y[2].title, "b".to_owned());
        assert_eq!(old.y[0].values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(old.y[1].values.len(), 5);
        assert!(old.y[1].values[0].is_nan());
        assert!(old.y[1].values[1].is_nan());
        assert!(old.y[1].values[2].is_nan());
        assert_eq!(&old.y[1].values[3..5], &[6.0, 7.0]);

        assert_eq!(old.y[2].values.len(), 5);
        assert_eq!(&old.y[2].values[0..3], &[2.0, 3.0, 4.0]);
        assert!(old.y[2].values[3].is_nan());
        assert!(old.y[2].values[4].is_nan());
    }
        #[test]
        fn append_set_to_empty() {
            let mut old = SeriesSet {
                x: None,
                y: vec![
                ],
            };
    
            let new = SeriesSet {
                x: None,
                y: vec![
                    Series {
                        title: "a".to_owned(),
                        values: vec![4.0, 5.0],
                    },
                    Series {
                        title: "c".to_owned(),
                        values: vec![6.0, 7.0],
                    },
                ],
            };
    
            old.append_set(new);
    
            assert_eq!(old.y.len(), 2);
            assert_eq!(old.y[1].title, "a".to_owned());
            assert_eq!(old.y[0].title, "c".to_owned());
            assert_eq!(old.y[1].values, vec![4.0, 5.0]);
            assert_eq!(old.y[0].values, vec![6.0, 7.0]);
      }
}
