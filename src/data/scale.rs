use crate::data::metric_parse::metric_parse;
use std::f64;

#[derive(Debug)]
pub enum ScaleError {
    EmptyDomain(f64, f64, f64),
    BadDomainConfig(String),
    NumberParse(std::num::ParseFloatError),
    BadFormat(String),
}

impl From<std::num::ParseFloatError> for ScaleError {
    fn from(err: std::num::ParseFloatError) -> ScaleError {
        ScaleError::NumberParse(err)
    }
}

impl std::fmt::Display for ScaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ScaleError::EmptyDomain(a, b, c) => write!(
                f,
                "Scale domain error: {}, {}, {} has empty interval.",
                a, b, c
            ),
            ScaleError::BadDomainConfig(ref s) => write!(f, "Scale domain error: {}", s),
            ScaleError::NumberParse(ref s) => write!(f, "Unable to parse: {}", s),
            ScaleError::BadFormat(ref s) => write!(f, "Bad Format: {}", s),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scale {
    a: f64,
    b: f64,
    c: f64,
}

impl Scale {
    // linear transform from
    // [a; b] -> [-1; 0] and
    // [b; c] -> [0; 1]
    pub fn new(a: f64, b: f64, c: f64) -> Result<Scale, ScaleError> {
        if a == b || b == c {
            return Err(ScaleError::EmptyDomain(a, b, c));
        }
        Ok(Scale { a, b, c })
    }

    // [a; b] -> [0; 1]
    pub fn new_positive(a: f64, b: f64) -> Result<Scale, ScaleError> {
        Scale::new(2.0 * a - b, a, b)
    }

    // [a; b] -> [-1; 0]
    pub fn new_negative(a: f64, b: f64) -> Result<Scale, ScaleError> {
        Scale::new(a, b, 2.0 * b - a)
    }

    // Find 'decent' scale for given list of values.
    // The logic is following:
    //  - if there's no data at all, return identity map;
    //  - if there's both negative and positive number, map [mn; 0; mx] -> [-1; 0; 1]
    //  - if only negative or positive number present, map [mn; 0] -> [-1; 0] OR [0; mx] -> [0; 1]

    // Error here would indicate a bug in a program.
    pub fn auto(v: &[f64]) -> Scale {
        match v
            .iter()
            .filter(|x| x.is_finite())
            .fold(None, |acc, x| match acc {
                None => Some((*x, *x)),
                Some((mn, mx)) => Some((mn.min(*x), mx.max(*x))),
            }) {
            None => Scale::new(-1.0, 0.0, 1.0).unwrap(), // identity mapping
            Some((mn, mx)) => {
                // both positive and negative numbers
                if mn * mx < 0.0 {
                    return Scale::new(mn, 0.0, mx).unwrap();
                }
                // negative map:
                if mn < 0.0 {
                    return Scale::new_negative(mn, 0.0).unwrap();
                }
                if mx > 0.0 {
                    return Scale::new_positive(0.0, mx).unwrap();
                }
                Scale::new(-1.0, 0.0, 1.0).unwrap()
            }
        }
    }

    fn transform(from: (f64, f64), to: (f64, f64), v: f64) -> f64 {
        to.0 + (to.1 - to.0) * (v - from.0) / (from.1 - from.0)
    }

    pub fn run(&self, v: f64) -> f64 {
        if v < self.b {
            Scale::transform((self.a, self.b), (-1.0, 0.0), v)
        } else {
            Scale::transform((self.b, self.c), (0.0, 1.0), v)
        }
    }

    pub fn from_config(config: &str) -> Result<Scale, ScaleError> {
        let v: Result<Vec<f64>, std::num::ParseFloatError> =
            config.split("..").map(|v| metric_parse(v)).collect();
        let v = v?;
        match v.len() {
            1 => Ok(Scale::new_positive(0.0, v[0])?),
            2 => Ok(Scale::new_positive(v[0], v[1])?),
            3 => Ok(Scale::new(v[0], v[1], v[2])?),
            _ => Err(ScaleError::BadDomainConfig(config.to_string())),
        }
    }
}

impl Default for Scale {
    fn default() -> Scale {
        Scale::new(-1.0, 0.0, 1.0).unwrap()
    }
}

#[derive(Debug)]
pub struct Scales {
    scales: Vec<(String, Scale)>,
}

impl Scales {
    // example config format:  250.0..1000..2000,qps:100000,cpu:0..100
    pub fn from_config(conf: &str) -> Result<Scales, ScaleError> {
        let mut catch_all: Option<(String, Scale)> = None;
        let scales: Result<Vec<Option<(String, Scale)>>, ScaleError> = conf
            .split(',')
            .map(|s| {
                let parts = s.split(':').collect::<Vec<&str>>();

                match parts.len() {
                    1 => {
                        catch_all = Some(("".to_owned(), Scale::from_config(&parts[0])?));
                        Ok(None)
                    }
                    2 => Ok(Some((parts[0].to_owned(), Scale::from_config(&parts[1])?))),
                    _ => Err(ScaleError::BadFormat(conf.to_owned())),
                }
            })
            .collect();

        let mut res: Vec<(String, Scale)> = scales?.into_iter().filter_map(|v| v).collect();

        // catch all is last
        if let Some(v) = catch_all {
            res.push(v);
        }

        Ok(Scales { scales: res })
    }

    // finds first one which matches the pattern
    pub fn pick(&self, title: &str) -> Option<Scale> {
        match self.scales.iter().find(|&(p, _)| title.contains(p)) {
            Some((_, scale)) => Some(*scale),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn scale_default() {
        assert_eq!(Scale::default().run(1.0), 1.0);
    }

    #[test]
    fn parse_default() {
        assert_eq!(
            Scales::from_config("1..2..3,x:2..3..4")
                .unwrap()
                .scales
                .len(),
            2
        );
    }

    #[test]
    fn scale_failures() {
        assert!(Scale::new(0.0, 0.0, 1.0).is_err());
        assert!(Scale::new_positive(1.0, 1.0).is_err());
        assert!(Scale::new_negative(10.0, 10.0).is_err());
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
            Scales::from_config("1..2..qq"),
            Err(ScaleError::NumberParse(_))
        );
        assert_match!(
            Scales::from_config("1..2..3..4"),
            Err(ScaleError::BadDomainConfig(_))
        );
        assert_match!(Scales::from_config("xyz"), Err(ScaleError::NumberParse(_)));
    }

    #[test]
    fn run0() {
        let s = Scale::new(-10.0, 0.0, 10.0).unwrap();
        assert_approx_eq!(s.run(0.0), 0.0);
        assert_approx_eq!(s.run(-1.0), -0.1);
        assert_approx_eq!(s.run(2.0), 0.2);
        assert_approx_eq!(s.run(20.0), 2.0);
    }

    #[test]
    fn parse_one_scale_and_run() {
        // maps [0; 10k] -> [0; 1]
        let scale10k = Scale::from_config("10k").unwrap();
        assert_approx_eq!(scale10k.a, -10000.0);
        assert_approx_eq!(scale10k.b, 0.0);
        assert_approx_eq!(scale10k.c, 10000.0);

        assert_approx_eq!(scale10k.run(-5000.0), -0.5);
        assert_approx_eq!(scale10k.run(25000.0), 2.5);

        // maps [-1k; 1k] -> [0; 1]
        let scale1k = Scale::from_config("-1k..1k").unwrap();
        assert_approx_eq!(scale1k.a, -3000.0);
        assert_approx_eq!(scale1k.b, -1000.0);
        assert_approx_eq!(scale1k.c, 1000.0);
        assert_approx_eq!(scale1k.run(-5000.0), -2.0);
        assert_approx_eq!(scale1k.run(0.0), 0.5);

        // maps [-1m; 0] -> [-1; 0] and [0; 1m] -> [0; 1]
        let scale1m = Scale::from_config("-1m..0..1m").unwrap();
        assert_approx_eq!(scale1m.a, -1000000.0);
        assert_approx_eq!(scale1m.b, 0.0);
        assert_approx_eq!(scale1m.c, 1000000.0);
        assert_approx_eq!(scale1m.run(-200000.0), -0.2);
        assert_approx_eq!(scale1m.run(1000000.0), 1.0);

        // maps [95;100] -> [-1; 0] and [100; 105] -> [0; 1]
        let scale100 = Scale::from_config("95..100..105").unwrap();
        assert_approx_eq!(scale100.a, 95.0);
        assert_approx_eq!(scale100.b, 100.0);
        assert_approx_eq!(scale100.c, 105.0);
        assert_approx_eq!(scale100.run(101.0), 0.2);
        assert_approx_eq!(scale100.run(96.0), -0.8);
    }

    #[test]
    fn parse_scales_and_run() {
        let scales = Scales::from_config("-100..0..100,x:-100..0..1000,z:500,w:-200..200").unwrap();
        assert_approx_eq!(scales.pick("x").unwrap().run(10.0), 0.01);
        assert_approx_eq!(scales.pick("y").unwrap().run(10.0), 0.1);
        assert_approx_eq!(scales.pick("y").unwrap().run(-10.0), -0.1);
        assert_approx_eq!(scales.pick("z").unwrap().run(250.0), 0.5);
        assert_approx_eq!(scales.pick("z").unwrap().run(-250.0), -0.5);
        assert_approx_eq!(scales.pick("w").unwrap().run(-100.0), 0.25);
        assert_approx_eq!(scales.pick("w").unwrap().run(-300.0), -0.25);
    }
}
