// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

/// Parse a duration string like "30s", "10m", "1h" into a `std::time::Duration`.
pub(crate) fn parse_duration(s: &str) -> Result<std::time::Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration is empty".to_string());
    }

    let lower = s.to_ascii_lowercase();
    let (num_str, unit) = match lower.chars().last() {
        Some('s') => (&lower[..lower.len() - 1], 's'),
        Some('m') => (&lower[..lower.len() - 1], 'm'),
        Some('h') => (&lower[..lower.len() - 1], 'h'),
        _ => return Err("duration must end with s, m, or h".to_string()),
    };

    let value: u64 = num_str
        .parse()
        .map_err(|_| "duration number is invalid".to_string())?;

    let secs = match unit {
        's' => value,
        'm' => value.saturating_mul(60),
        'h' => value.saturating_mul(3600),
        _ => return Err("duration unit is invalid".to_string()),
    };

    Ok(std::time::Duration::from_secs(secs))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_seconds() {
        assert_eq!(
            parse_duration("30s").unwrap(),
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn valid_minutes() {
        assert_eq!(
            parse_duration("10m").unwrap(),
            std::time::Duration::from_secs(600)
        );
    }

    #[test]
    fn valid_hours() {
        assert_eq!(
            parse_duration("1h").unwrap(),
            std::time::Duration::from_secs(3600)
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            parse_duration("5M").unwrap(),
            std::time::Duration::from_secs(300)
        );
        assert_eq!(
            parse_duration("5H").unwrap(),
            std::time::Duration::from_secs(18000)
        );
        assert_eq!(
            parse_duration("5S").unwrap(),
            std::time::Duration::from_secs(5)
        );
    }

    #[test]
    fn zero_duration_parses_but_caller_rejects() {
        // parse_duration itself accepts zero; callers enforce > 0.
        assert_eq!(
            parse_duration("0s").unwrap(),
            std::time::Duration::from_secs(0)
        );
    }

    #[test]
    fn empty_rejected() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn whitespace_only_rejected() {
        assert!(parse_duration("   ").is_err());
    }

    #[test]
    fn missing_unit_rejected() {
        assert!(parse_duration("30").is_err());
    }

    #[test]
    fn invalid_number_rejected() {
        assert!(parse_duration("abcs").is_err());
        assert!(parse_duration("-5m").is_err());
    }

    #[test]
    fn large_values_saturate() {
        // u64::MAX seconds would overflow, but saturating_mul prevents panic.
        let result = parse_duration("99999999999999h");
        assert!(result.is_ok());
    }
}
