use std::time::Duration;

/// Parse a human time string like "40s", "5m", "5.5h" or "1h30m".
/// A plain number with no unit means minutes.
pub fn parse_duration(input: &str) -> Result<Duration, String> {
    let text = input.trim().to_lowercase();
    if text.is_empty() {
        return Err("time is empty".to_string());
    }

    // Plain number: treat it as minutes.
    if let Ok(minutes) = text.parse::<f64>() {
        return to_duration(minutes * 60.0, input);
    }

    let mut total_secs = 0.0;
    let mut chars = text.chars().peekable();

    while chars.peek().is_some() {
        // Read the number part, like "5" or "5.5".
        let mut number = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                number.push(c);
                chars.next();
            } else {
                break;
            }
        }
        let value: f64 = number
            .parse()
            .map_err(|_| format!("expected a number in \"{input}\""))?;

        // Read the unit part, like "h", "m" or "s".
        let mut unit = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() {
                unit.push(c);
                chars.next();
            } else {
                break;
            }
        }
        let secs_per_unit = match unit.as_str() {
            "h" => 3600.0,
            "m" => 60.0,
            "s" => 1.0,
            "" => return Err(format!("missing unit after {number} in \"{input}\"")),
            other => return Err(format!("unknown unit \"{other}\" (use h, m or s)")),
        };
        total_secs += value * secs_per_unit;
    }

    to_duration(total_secs, input)
}

fn to_duration(secs: f64, input: &str) -> Result<Duration, String> {
    if !secs.is_finite() || secs <= 0.0 {
        return Err(format!("time must be more than zero: \"{input}\""));
    }
    Ok(Duration::from_secs_f64(secs))
}

/// Format a duration as "MM:SS", or "H:MM:SS" when it is an hour or more.
pub fn format_clock(d: Duration) -> String {
    let total = d.as_secs();
    let (h, m, s) = (total / 3600, total % 3600 / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(input: &str) -> f64 {
        parse_duration(input).unwrap().as_secs_f64()
    }

    #[test]
    fn single_units() {
        assert_eq!(secs("40s"), 40.0);
        assert_eq!(secs("5m"), 300.0);
        assert_eq!(secs("5.5h"), 19800.0);
    }

    #[test]
    fn combined_units() {
        assert_eq!(secs("1h30m"), 5400.0);
        assert_eq!(secs("2m30s"), 150.0);
    }

    #[test]
    fn plain_number_is_minutes() {
        assert_eq!(secs("25"), 1500.0);
        assert_eq!(secs("0.5"), 30.0);
    }

    #[test]
    fn spaces_and_case() {
        assert_eq!(secs("  10M "), 600.0);
    }

    #[test]
    fn bad_input() {
        for bad in ["", "abc", "5x", "m", "0s", "-5m", "1.2.3m", "5m10"] {
            assert!(parse_duration(bad).is_err(), "should fail: {bad:?}");
        }
    }

    #[test]
    fn clock_format() {
        assert_eq!(format_clock(Duration::from_secs(65)), "01:05");
        assert_eq!(format_clock(Duration::from_secs(3725)), "1:02:05");
    }
}
