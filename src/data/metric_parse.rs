fn base<'a>(v: &'a str) -> &'a str {
    &v[0..v.len() - 1]
}

pub fn metric_parse(s: &str) -> Result<f64, std::num::ParseFloatError> {
    let s = s.trim();
    let (exponent, mantissa) = match s.to_uppercase().chars().last() {
        Some('K') => (1.0e3, base(s)),
        Some('M') => (1.0e6, base(s)),
        Some('G') => (1.0e9, base(s)),
        Some('T') => (1.0e12, base(s)),
        _ => (1.0, s),
    };

    mantissa.parse::<f64>().map(|m| m * exponent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_parse_test() {
        assert_eq!(1234.0, metric_parse("1.234k").unwrap());
        assert_eq!(1234000.0, metric_parse("1.234M").unwrap());
        assert_eq!(123.0, metric_parse("123").unwrap());
        assert!(metric_parse("123OLOLOL").is_err());
    }
}
