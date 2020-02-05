use crate::data::scale::{min_max, Scale, ScaleError, Scales};
use crate::data::series::Series;

use std::collections::HashMap;
use std::f64;

#[derive(Debug)]
enum ScaleType {
    Static(Scale),
    Auto,
}

impl ScaleType {
    pub fn new(conf: &str) -> Result<ScaleType, ScaleError> {
        if conf == "auto" {
            Ok(ScaleType::Auto)
        } else {
            Ok(ScaleType::Static(Scale::from_config(conf)?))
        }
    }
}

#[derive(Debug)]
pub struct ScaleConfig {
    pattern: String,
    config: ScaleType,
}

impl ScaleConfig {
    pub fn new(pattern: &str, conf: &str) -> Result<ScaleConfig, ScaleError> {
        Ok(ScaleConfig {
            pattern: pattern.to_owned(),
            config: ScaleType::new(conf)?,
        })
    }

    fn make_scale(&self, bounds: &HashMap<String, (f64, f64)>) -> (String, Scale) {
        match self.config {
            ScaleType::Auto => {
                if let Some((mn, mx)) = bounds.get(&self.pattern) {
                    (self.pattern.clone(), Scale::from_min_max(*mn, *mx).unwrap())
                } else {
                    (self.pattern.clone(), Scale::default())
                }
            }
            ScaleType::Static(scale) => (self.pattern.to_owned(), scale),
        }
    }
}

// ScalesConfig reads the config passed and create a list of 'ScaleConfig's.
// ScaleConfig might be a fully-defined static Scale, or an autoscaling group.
#[derive(Debug)]
pub struct ScalesConfig {
    entries: Vec<ScaleConfig>,
}

impl ScalesConfig {
    pub fn new(conf: &str) -> Result<ScalesConfig, ScaleError> {
        let mut wildcard: Option<ScaleConfig> = None;
        let scales: Result<Vec<Option<ScaleConfig>>, ScaleError> = conf
            .split(',')
            .map(|s| {
                let parts = s.split(':').collect::<Vec<&str>>();

                match parts.len() {
                    1 => {
                        wildcard = Some(ScaleConfig::new("", &parts[0])?);
                        Ok(None)
                    }
                    2 => Ok(Some(ScaleConfig::new(&parts[0], &parts[1])?)),
                    _ => Err(ScaleError::BadFormat(conf.to_owned())),
                }
            })
            .collect();

        let mut res: Vec<ScaleConfig> = scales?.into_iter().filter_map(|v| v).collect();

        // wildcard is last
        if let Some(v) = wildcard {
            res.push(v);
        }

        Ok(ScalesConfig { entries: res })
    }

    // finds first matching scale config for a series title,
    // and, if it's 'autoscale' returns it. If the match is
    // not autoscale, or no result was found, None is returned.
    fn find_auto(&self, title: &str) -> Option<&ScaleConfig> {
        self.entries
            .iter()
            .find(|c| title.contains(&c.pattern))
            .filter(|c| match c.config {
                ScaleType::Auto => true,
                _ => false,
            })
    }

    // for each autoscale config, computes min/max values in the data.
    fn bounds(&self, series: &[Series]) -> HashMap<String, (f64, f64)> {
        let mut bounds = HashMap::new();
        series.iter().for_each(|s| {
            // if it's autoscale matching the series
            if let Some(scale_config) = self.find_auto(&s.title) {
                // and if there's valid min and max values for the series
                if let Some((mn, mx)) = min_max(&s.values) {
                    bounds
                        .entry(scale_config.pattern.clone())
                        .and_modify(|v: &mut (f64, f64)| {
                            v.0 = v.0.min(mn);
                            v.1 = v.1.max(mx);
                        })
                        .or_insert((mn, mx));
                }
            }
        });
        bounds
    }

    // Transforms autoscaling groups to completely defined scales
    // given the data set.
    pub fn materialize(&self, series: &[Series]) -> Scales {
        Scales::with_scales({
            let bounds = self.bounds(series);
            self.entries.iter().map(|s| s.make_scale(&bounds)).collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn parse_default() {
        assert_eq!(
            ScalesConfig::new("1..2..3,x:2..3..4")
                .unwrap()
                .entries
                .len(),
            2
        );
    }

    macro_rules! assert_match {
        ($what:expr, $($pattern:tt)+) => {
            if let $($pattern)+ = $what {} else {
                panic!("assertion failed: `{}` does not match `{}`", stringify!($what), stringify!($($pattern)+));
            }
        }
    }

    #[test]
    fn parse_failures() {
        assert_match!(
            ScalesConfig::new("1..2..qq"),
            Err(ScaleError::NumberParse(_))
        );
        assert_match!(
            ScalesConfig::new("1..2..3..4"),
            Err(ScaleError::BadDomainConfig(_))
        );
        assert_match!(ScalesConfig::new("xyz"), Err(ScaleError::NumberParse(_)));
    }

    #[test]
    fn parse_scales_and_run() {
        let scales = ScalesConfig::new("-100..0..100,x:-100..0..1000,z:500,w:-200..200").unwrap();
        let scales = scales.materialize(&vec![]);
        assert_approx_eq!(scales.pick("x").unwrap().run(10.0), 0.01);
        assert_approx_eq!(scales.pick("y").unwrap().run(10.0), 0.1);
        assert_approx_eq!(scales.pick("y").unwrap().run(-10.0), -0.1);
        assert_approx_eq!(scales.pick("z").unwrap().run(250.0), 0.5);
        assert_approx_eq!(scales.pick("z").unwrap().run(-250.0), -0.5);
        assert_approx_eq!(scales.pick("w").unwrap().run(-100.0), 0.25);
        assert_approx_eq!(scales.pick("w").unwrap().run(-300.0), -0.25);
    }

    #[test]
    fn materialized() {
        let s = vec![
            Series {
                title: "cpu1".to_owned(),
                values: vec![0.0, 10.0, 11.0, 9.0],
            },
            Series {
                title: "cpu2".to_owned(),
                values: vec![0.0, 100.0, 11.0, 99.0],
            },
            Series {
                title: "ram_free_mb".to_owned(),
                values: vec![1111.0, 999.0, 888.0, 99.0],
            },
        ];
        let scales = ScalesConfig::new("cpu:auto").unwrap();
        let scales = scales.materialize(&s);
        // in this case, cpu will be auto, ram will be also 'auto' but separate

        let cpu_scale = scales.pick("cpu1").unwrap();
        let (a, b, c) = cpu_scale.to_tuple();
        assert_approx_eq!(a, -100.0);
        assert_approx_eq!(b, 0.0);
        assert_approx_eq!(c, 100.0);
        let cpu_scale = scales.pick("cpu2").unwrap();
        let (a, b, c) = cpu_scale.to_tuple();
        assert_approx_eq!(a, -100.0);
        assert_approx_eq!(b, 0.0);
        assert_approx_eq!(c, 100.0);
    }
}
