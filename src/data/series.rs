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

    pub fn append(&mut self, mut other: SeriesSet) {
        if let Some((_, xx)) = other.x.as_mut() {
            if let Some((_, x)) = self.x.as_mut() {
                x.append(xx);
            } else {
                self.x = other.x;
            }
        }
        if self.y.is_empty() {
            self.y = other.y;
        } else {
            self.y
                .iter_mut()
                .zip(other.y.iter_mut())
                .for_each(|(y, yy)| {
                    y.values.append(&mut yy.values);
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_both_present() {
        let mut current = SeriesSet {
            x: None,
            y: vec![
                Series {
                    title: "a".to_owned(),
                    values: vec![1.0],
                },
                Series {
                    title: "b".to_owned(),
                    values: vec![2.0],
                },
            ],
        };
        let new = SeriesSet {
            x: None,
            y: vec![
                Series {
                    title: "a".to_owned(),
                    values: vec![3.0],
                },
                Series {
                    title: "b".to_owned(),
                    values: vec![4.0],
                },
            ],
        };
        current.append(new);
        assert_eq!(current.y.len(), 2);
        assert_eq!(current.y[0].values, vec![1.0, 3.0]);
        assert_eq!(current.y[1].values, vec![2.0, 4.0]);
    }

    #[test]
    fn test_append_empty_old() {
        let mut current = SeriesSet { x: None, y: vec![] };
        let new = SeriesSet {
            x: Some(("time".to_owned(), vec!["22:22".to_owned()])),
            y: vec![
                Series {
                    title: "a".to_owned(),
                    values: vec![3.0],
                },
                Series {
                    title: "b".to_owned(),
                    values: vec![4.0],
                },
            ],
        };
        current.append(new);
        assert_eq!(current.y.len(), 2);
        assert_eq!(current.y[0].values, vec![3.0]);
        assert_eq!(current.y[1].values, vec![4.0]);
        assert_eq!(
            current.x,
            Some(("time".to_owned(), vec!["22:22".to_owned()]))
        );
    }
}
