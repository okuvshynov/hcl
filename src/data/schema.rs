use crate::app::settings::Column;
use crate::data::series::{Series, SeriesSet, Slice};

struct ColumnSchema {
    title: String,
    index: usize,
}

impl ColumnSchema {
    pub fn new(title: String, index: usize) -> ColumnSchema {
        ColumnSchema { title, index }
    }
}

/// Schema represents the way input data is transformed to internal format.
/// At the moment, schema has only the definition of two special fields: X and Epoch.
/// Every other field from the input will become a data series.
pub struct Schema {
    x: Option<ColumnSchema>,
    // titles should be also stored here.
    titles: Vec<String>,
}

impl Schema {
    // parse string here?

    fn default() -> Schema {
        Schema {
            x: None,
            titles: vec![],
        }
    }

    pub fn from_title_range(x: Column, titles: &[String]) -> Schema {
        let mut res = Schema::default();

        titles.iter().zip(0..).for_each(|(t, i)| {
            if x.matches(t, i) {
                res.x = Some(ColumnSchema::new(t.to_owned(), i));
            } else {
                res.titles.push(t.to_owned());
            }
        });

        res
    }

    pub fn from_titles(x: Column, titles: &str) -> Schema {
        let mut res = Schema::default();

        titles.split(',').zip(0..).for_each(|(t, i)| {
            if x.matches(t, i) {
                res.x = Some(ColumnSchema::new(t.to_owned(), i));
            } else {
                res.titles.push(t.to_owned());
            }
        });

        res
    }

    /// Returns a stub of SeriesSet, with correct number of
    /// empty series.
    pub fn empty_set(&self) -> SeriesSet {
        SeriesSet {
            x: match self.x {
                Some(ref x) => Some((x.title.clone(), vec![])),
                _ => None,
            },
            y: self.titles.iter().map(|t| Series::with_title(t)).collect(),
        }
    }

    /// Formats a row of input data as a slice.
    /// Slice can be appended to a SeriesSet.
    pub fn slice(&self, slice: &str) -> Slice {
        let mut res = Slice::default();
        slice
            .split(',')
            .enumerate()
            .for_each(|(i, v)| match &self.x {
                Some(x) if x.index == i => res.x = Some(v.to_owned()),
                _ => res.y.push(v.trim().parse::<f64>().unwrap_or(std::f64::NAN)),
            });
        res
    }

    pub fn slice_from_range(&self, slice: &[String]) -> Slice {
        let mut res = Slice::default();
        slice.iter().enumerate().for_each(|(i, v)| match &self.x {
            Some(x) if x.index == i => res.x = Some(v.to_owned()),
            _ => res.y.push(v.trim().parse::<f64>().unwrap_or(std::f64::NAN)),
        });
        res
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema() {
        let schema = Schema::from_titles(Column::None, "a,b,c");
        let s = schema.empty_set();
        assert_eq!(s.x, None);
        assert_eq!(s.y.len(), 3);
        assert_eq!(s.y[0].title, "a");
        assert_eq!(s.y[1].title, "b");
        assert_eq!(s.y[2].title, "c");

        let slice = schema.slice("1,2,3");
        assert_eq!(slice.x, None);
        assert_eq!(slice.y, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_x() {
        let schema = Schema::from_titles(Column::Index(0), "a,b,c");
        let s = schema.empty_set();
        assert_eq!(s.x, Some(("a".to_owned(), vec![])));
        assert_eq!(s.y.len(), 2);
        assert_eq!(s.y[0].title, "b");

        let slice = schema.slice("1,2,3");
        assert_eq!(slice.x, Some("1".to_owned()));
        assert_eq!(slice.y, vec![2.0, 3.0]);
    }
}
