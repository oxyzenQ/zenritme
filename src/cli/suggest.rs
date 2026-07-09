// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

//! Did-you-mean suggestions via Levenshtein edit distance.
//!
//! Used by the CLI to suggest the closest valid theme/view name when a user
//! mistypes `--theme` or `--view`.  Pure functions, no I/O, fully testable.

/// Maximum edit distance to consider a candidate a "close enough" match.
const THRESHOLD: usize = 3;

/// Compute the Levenshtein edit distance between two strings.
///
/// Standard O(m·n) dynamic programming.  Returns the minimum number of
/// single-character insertions, deletions, or substitutions needed to
/// transform `a` into `b`.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // Single-row DP — prev[j] = distance(a[..i-1], b[..j])
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Returns the closest candidate by edit distance, if within `THRESHOLD`.
///
/// Case-insensitive comparison (input is lowercased before measuring).
/// Ties are broken by the first candidate encountered (stable).
pub fn closest(input: &str, candidates: &[&str]) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for &cand in candidates {
        let dist = levenshtein(&lower, &cand.to_ascii_lowercase());
        if dist <= THRESHOLD && best.is_none_or(|(d, _)| dist < d) {
            best = Some((dist, cand));
        }
    }
    best.map(|(_, c)| c.to_string())
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── levenshtein ───────────────────────────────────────────────────────

    #[test]
    fn lev_identical_strings() {
        assert_eq!(levenshtein("ember", "ember"), 0);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn lev_empty_a() {
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn lev_empty_b() {
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn lev_single_substitution() {
        assert_eq!(levenshtein("ember", "embor"), 1);
    }

    #[test]
    fn lev_single_insertion() {
        assert_eq!(levenshtein("ember", "emberr"), 1);
    }

    #[test]
    fn lev_single_deletion() {
        assert_eq!(levenshtein("ember", "embe"), 1);
    }

    #[test]
    fn lev_completely_different() {
        assert_eq!(levenshtein("abc", "xyz"), 3);
    }

    #[test]
    fn lev_unicode_safe() {
        // é is a single char — distance 1 from e.
        assert_eq!(levenshtein("café", "cafe"), 1);
    }

    // ── closest ───────────────────────────────────────────────────────────

    #[test]
    fn closest_exact_match() {
        assert_eq!(
            closest("ember", THEME_CANDIDATES),
            Some("ember".to_string())
        );
    }

    #[test]
    fn closest_typo_one_char() {
        assert_eq!(
            closest("embar", THEME_CANDIDATES),
            Some("ember".to_string())
        );
    }

    #[test]
    fn closest_case_insensitive() {
        assert_eq!(
            closest("EMBER", THEME_CANDIDATES),
            Some("ember".to_string())
        );
        assert_eq!(closest("Orbit", VIEW_CANDIDATES), Some("orbit".to_string()));
    }

    #[test]
    fn closest_prefix_typo() {
        assert_eq!(closest("mni", THEME_CANDIDATES), Some("mono".to_string()));
    }

    #[test]
    fn closest_returns_none_when_too_far() {
        assert_eq!(closest("zzzzzzz", THEME_CANDIDATES), None);
    }

    #[test]
    fn closest_returns_none_when_empty() {
        assert_eq!(closest("", THEME_CANDIDATES), None);
    }

    #[test]
    fn closest_returns_none_when_no_candidates() {
        assert_eq!(closest("ember", &[]), None);
    }

    #[test]
    fn closest_tron_family_disambiguation() {
        // "tron-cyan" should win over "tron" for input "tron-cya".
        let got = closest("tron-cya", THEME_CANDIDATES);
        assert_eq!(got.as_deref(), Some("tron-cyan"));
    }

    // ── Test fixtures ─────────────────────────────────────────────────────

    const THEME_CANDIDATES: &[&str] = &[
        "void",
        "ember",
        "aura",
        "forest",
        "tron",
        "tron-green",
        "tron-cyan",
        "tron-orange",
        "tron-red",
        "tron-yellow",
        "mono",
    ];

    const VIEW_CANDIDATES: &[&str] = &["minimal", "orbit", "cinematic", "tron"];
}
