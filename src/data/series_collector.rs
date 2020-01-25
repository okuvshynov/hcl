use crate::app::settings::XColumn;
use crate::data::series::{Series, SeriesSet};

pub struct SeriesCollector {
    pub xi: Option<usize>,
    data: SeriesSet,
}

impl SeriesCollector {
    pub fn default() -> SeriesCollector {
        SeriesCollector {
            data: SeriesSet::default(),
            xi: None,
        }
    }

    pub fn from_titles<'a, I>(titles: I, x: &XColumn) -> SeriesCollector
    where
        I: Iterator<Item = &'a str>,
    {
        let mut res = SeriesCollector::default();
        titles.zip(0..).for_each(|(t, i)| {
            if match x {
                XColumn::None => false,
                XColumn::Index(xi) => i == *xi,
                XColumn::Title(title) => title == t,
            } {
                res.xi = Some(i);
                res.data.x = Some((t.to_string(), vec![]));
            } else {
                res.data.y.push(Series::with_title(t));
            }
        });

        res
    }

    pub fn one_row<'a, I>(&mut self, values: I)
    where
        I: Iterator<Item = &'a str>,
    {
        if let Some((_, x)) = self.data.x.as_mut() {
            x.clear();
        }
        self.data.y.iter_mut().for_each(|y| y.values.clear());
        self.append(values);
    }

    pub fn append<'a, I>(&mut self, mut values: I)
    where
        I: Iterator<Item = &'a str>,
    {
        // trying to parse exactly as many elements as titles;
        // NAN is used for missing/failed parsing ones
        let mut vi = 0;
        let mut xv: Option<&str> = None;

        let xi = self.xi;

        self.data.y.iter_mut().for_each(|l| {
            if xi.map_or(false, |xi| xi == vi) {
                // This is X column;
                xv = values.next();
            }
            l.values.push(
                values
                    .next()
                    .unwrap_or("")
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(std::f64::NAN),
            );
            vi += 1;
        });

        // adding x value if it was there
        if let Some(xv) = xv {
            self.data.x.as_mut().map(|(_, v)| v.push(xv.to_string()));
        }
    }

    pub fn current(&self) -> SeriesSet {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_data() {
        let mut c =
            SeriesCollector::from_titles(vec!["a", "b", "c"].iter().map(|s| *s), &XColumn::None);
        c.append(vec!["0.1", "1.0", "-1"].iter().map(|s| *s));
        let hd = c.current();

        assert_eq!(hd.y.len(), 3);
        assert_eq!(hd.y[0].title, "a");
        assert_eq!(hd.y[0].values, vec![0.1]);
        assert_eq!(hd.y[1].title, "b");
        assert_eq!(hd.y[1].values, vec![1.0]);
        assert_eq!(hd.y[2].title, "c");
        assert_eq!(hd.y[2].values, vec![-1.0]);
    }

    #[test]
    fn extra_values() {
        let mut c =
            SeriesCollector::from_titles(vec!["a", "b", "c"].iter().map(|s| *s), &XColumn::None);
        c.append(vec!["1.0", "-1", "100", "00", "xxx"].iter().map(|s| *s));
        let hd = c.current();

        assert_eq!(hd.y.len(), 3);
        assert_eq!(hd.y[0].title, "a");
        assert_eq!(hd.y[0].values, vec![1.0]);
        assert_eq!(hd.y[1].title, "b");
        assert_eq!(hd.y[1].values, vec![-1.0]);
        assert_eq!(hd.y[2].title, "c");
        assert_eq!(hd.y[2].values, vec![100.0]);
    }

    #[test]
    fn missing_values() {
        let mut c =
            SeriesCollector::from_titles(vec!["a", "b", "c"].iter().map(|s| *s), &XColumn::None);
        c.append(vec!["1.0", "-1"].iter().map(|s| *s));
        let hd = c.current();

        assert_eq!(hd.y.len(), 3);
        assert_eq!(hd.y[0].title, "a");
        assert_eq!(hd.y[0].values, vec![1.0]);
        assert_eq!(hd.y[1].title, "b");
        assert_eq!(hd.y[1].values, vec![-1.0]);
        assert_eq!(hd.y[2].title, "c");
        assert!(hd.y[2].values[0].is_nan());
    }

    #[test]
    fn parse_errors() {
        let mut c =
            SeriesCollector::from_titles(vec!["a", "b", "c"].iter().map(|s| *s), &XColumn::None);
        c.append(vec!["x", "1.0", "-1"].iter().map(|s| *s));
        let hd = c.current();

        assert_eq!(hd.y.len(), 3);
        assert_eq!(hd.y[0].title, "a");
        assert!(hd.y[0].values[0].is_nan());
        assert_eq!(hd.y[1].title, "b");
        assert_eq!(hd.y[1].values, vec![1.0]);
        assert_eq!(hd.y[2].title, "c");
        assert_eq!(hd.y[2].values, vec![-1.0]);
    }

    #[test]
    fn x_column() {
        let mut c = SeriesCollector::from_titles(
            vec!["a", "b", "c"].iter().map(|s| *s),
            &XColumn::Index(1),
        );
        c.append(vec!["0.1", "1.0", "-1"].iter().map(|s| *s));
        let hd = c.current();

        assert_eq!(hd.y.len(), 2);
        assert_eq!(hd.y[0].title, "a");
        assert_eq!(hd.y[0].values, vec![0.1]);
        assert_eq!(hd.y[1].title, "c");
        assert_eq!(hd.y[1].values, vec![-1.0]);
        assert_eq!(hd.x, Some(("b".to_string(), vec!["1.0".to_string()])));
    }
}
