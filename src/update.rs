// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 Rezky Nightky

//! Safe, read-only update checker.
//!
//! This module provides `--check-update` functionality. It queries the GitHub
//! releases API via `curl`, parses the `tag_name` from the JSON response with
//! a minimal hand-rolled parser (no `serde_json`), and compares versions using
//! simple SemVer-ish logic.
//!
//! **Security properties:**
//! - Read-only: never downloads, installs, or replaces binaries.
//! - Never pipes remote content into a shell.
//! - Requires no authentication tokens.
//! - Network access is only triggered by `--check-update`; normal timer usage
//!   never touches the network.

use std::process::Command;

const GITHUB_API_URL: &str = "https://api.github.com/repos/oxyzenQ/zenritme/releases/latest";
const RELEASES_URL: &str = "https://github.com/oxyzenQ/zenritme/releases/latest";

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable,
    CurrentIsNewer,
    CurrentIsPrereleaseNewer,
}

/// Return value of [`compare_versions`]. Contains version strings and
/// the computed status. The `current` and `latest` fields are kept for
/// callers that want the raw strings; `check_update` prints its own output.
#[allow(dead_code)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub status: UpdateStatus,
}

// ─── Version comparison ───────────────────────────────────────────────────────

/// A parsed (major, minor, patch, pre) tuple.
/// `pre` is `None` for stable releases and `Some(label)` for prereleases.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
}

impl SemVer {
    fn parse(tag: &str) -> Option<Self> {
        let s = tag.strip_prefix('v').unwrap_or(tag);
        let (num_part, pre) = match s.split_once('-') {
            Some((n, p)) => (n, Some(p.to_string())),
            None => (s, None),
        };
        let mut parts = num_part.split('.');
        let major = parts.next()?.parse::<u64>().ok()?;
        let minor = parts.next()?.parse::<u64>().ok()?;
        let patch = parts.next()?.parse::<u64>().ok()?;
        // Reject trailing segments (e.g. "v1.2.3.4" is not valid SemVer here).
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        // Same major.minor.patch — stable (no pre) beats prerelease.
        match (&self.pre, &other.pre) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater, // stable > rc
            (Some(_), None) => std::cmp::Ordering::Less,    // rc < stable
            (Some(a), Some(b)) => a.cmp(b),                 // lexicographic
        }
    }
}

/// Compare current version against latest release and return the update status.
pub fn compare_versions(current: &str, latest: &str) -> UpdateInfo {
    let cur = SemVer::parse(current);
    let lat = SemVer::parse(latest);

    let status = match (&cur, &lat) {
        (Some(c), Some(l)) => {
            if c == l {
                // Exact match, but check if both have same pre-status.
                if c.pre.is_some() && l.pre.is_none() {
                    // Same numeric version but current is prerelease — treat as
                    // prerelease/newer line only if latest stable is numerically
                    // older or equal. Since they're equal here, current is
                    // prerelease of the latest stable.
                    UpdateStatus::CurrentIsPrereleaseNewer
                } else {
                    UpdateStatus::UpToDate
                }
            } else if c > l {
                if cur.as_ref().is_some_and(|v| v.pre.is_some()) {
                    UpdateStatus::CurrentIsPrereleaseNewer
                } else {
                    UpdateStatus::CurrentIsNewer
                }
            } else {
                UpdateStatus::UpdateAvailable
            }
        }
        _ => {
            // Cannot parse — assume update available to be safe.
            UpdateStatus::UpdateAvailable
        }
    };

    UpdateInfo {
        current: current.to_string(),
        latest: latest.to_string(),
        status,
    }
}

// ─── JSON parsing ────────────────────────────────────────────────────────────

/// Extract the value of `"tag_name"` from a GitHub API JSON response.
///
/// This is a minimal, safe parser that scans for the literal key `"tag_name"`
/// followed by a colon and a quoted string value. It does not deserialize the
/// full JSON — just plucks out the one field we need.
pub fn extract_tag_name(json: &str) -> Option<String> {
    // Find "tag_name" followed by colon.
    let key = "\"tag_name\"";
    let pos = json.find(key)?;
    let rest = &json[pos + key.len()..];

    // Skip whitespace and colon.
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();

    // Expect opening quote.
    let rest = rest.strip_prefix('"')?;

    // Read until closing quote (handle simple escapes).
    let mut value = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next()? {
            '"' => break,
            '\\' => match chars.next()? {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                'n' => value.push('\n'),
                't' => value.push('\t'),
                '/' => value.push('/'),
                c => {
                    value.push('\\');
                    value.push(c);
                }
            },
            c => value.push(c),
        }
    }

    Some(value)
}

// ─── Version normalization ────────────────────────────────────────────────────

/// Ensure a version string has exactly one leading `v`.
///
/// - `"1.3.0"`        → `"v1.3.0"`
/// - `"v1.3.0"`       → `"v1.3.0"`
/// - `"v2.0.0-rc.1"`  → `"v2.0.0-rc.1"`
/// - `""`             → `"v"`
pub fn normalize_version(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('v') {
        trimmed.to_string()
    } else {
        format!("v{trimmed}")
    }
}

// ─── Network fetch ───────────────────────────────────────────────────────────

/// curl exit codes that we map to human-readable messages.
/// See <https://curl.se/docs/manpage.html#EXIT-CODES>
fn interpret_curl_exit(code: i32) -> &'static str {
    match code {
        6 => "DNS resolution failed",
        7 => "connection refused",
        28 => "network request timed out",
        35 => "SSL/TLS handshake failed",
        _ => "network request failed",
    }
}

/// Classify an HTTP status code from the GitHub API response.
fn interpret_http_status(code: u16) -> &'static str {
    match code {
        404 => "no latest GitHub release found for oxyzenQ/zenritme",
        403 => "GitHub API request was rate-limited or forbidden",
        _ => "GitHub API returned an unexpected error",
    }
}

/// Run the full update check: fetch latest release from GitHub, compare versions,
/// and print the result. Returns `Ok(())` on success or `Err(reason)` on failure.
pub fn check_update(current_version: &str) -> Result<(), String> {
    // Use --write-out to capture the HTTP status code on a separate final line.
    // We intentionally do NOT use --fail so we can distinguish HTTP error codes.
    let output = Command::new("curl")
        .args([
            "--silent",
            "--max-time",
            "15",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "User-Agent: zenritme",
            "--write-out",
            "\n%{http_code}",
            GITHUB_API_URL,
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "curl is not available on PATH".to_string()
            } else {
                format!("failed to run curl: {e}")
            }
        })?;

    // Check curl's own exit code (network-level failure, not HTTP).
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        return Err(interpret_curl_exit(code).to_string());
    }

    // Split response: body is everything before the final newline,
    // HTTP status code is the last line.
    let raw = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = raw.rsplitn(2, '\n').collect();
    lines.reverse(); // [body, status_code]

    let http_code: u16 = lines
        .get(1)
        .and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(0);

    // Non-200 HTTP responses are mapped to human-readable messages.
    if http_code != 200 {
        return Err(interpret_http_status(http_code).to_string());
    }

    let body_owned =
        String::from_utf8(output.stdout).map_err(|_| "response was not valid UTF-8".to_string())?;
    // Re-extract body (before the trailing status line).
    let body_trimmed = if let Some(pos) = body_owned.rfind('\n') {
        &body_owned[..pos]
    } else {
        &body_owned
    };

    let latest_tag = extract_tag_name(body_trimmed)
        .ok_or_else(|| "could not parse latest release tag from GitHub response".to_string())?;

    let info = compare_versions(current_version, &latest_tag);

    let status_text = match &info.status {
        UpdateStatus::UpToDate => "up to date",
        UpdateStatus::UpdateAvailable => "update available",
        UpdateStatus::CurrentIsNewer => "current is newer than latest release",
        UpdateStatus::CurrentIsPrereleaseNewer => "current is a prerelease/newer line",
    };

    println!("Zenritme update check");
    let current_display = normalize_version(current_version);
    let latest_display = normalize_version(&latest_tag);

    println!("Current: {current_display}");
    println!("Latest:  {latest_display}");
    println!("Status:  {status_text}");
    println!("Source:  {RELEASES_URL}");

    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── SemVer parsing ─────────────────────────────────────────────────────

    #[test]
    fn semver_parse_stable() {
        let v = SemVer::parse("v1.3.0").unwrap();
        assert_eq!(
            v,
            SemVer {
                major: 1,
                minor: 3,
                patch: 0,
                pre: None
            }
        );
    }

    #[test]
    fn semver_parse_prerelease() {
        let v = SemVer::parse("v2.0.0-rc.1").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.pre.as_deref(), Some("rc.1"));
    }

    #[test]
    fn semver_parse_without_v_prefix() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(
            v,
            SemVer {
                major: 1,
                minor: 2,
                patch: 3,
                pre: None
            }
        );
    }

    #[test]
    fn semver_parse_invalid_returns_none() {
        assert!(SemVer::parse("").is_none());
        assert!(SemVer::parse("abc").is_none());
        assert!(SemVer::parse("1.2").is_none());
        assert!(SemVer::parse("v1.2.3.4").is_none());
    }

    // ── SemVer comparison ──────────────────────────────────────────────────

    #[test]
    fn compare_same_stable() {
        assert_eq!(SemVer::parse("v1.3.0"), SemVer::parse("v1.3.0"));
    }

    #[test]
    fn compare_stable_older_newer() {
        assert!(SemVer::parse("v1.3.0") < SemVer::parse("v2.0.0"));
        assert!(SemVer::parse("v1.2.9") < SemVer::parse("v1.3.0"));
        assert!(SemVer::parse("v1.3.0") < SemVer::parse("v1.3.1"));
    }

    #[test]
    fn compare_stable_beats_prerelease() {
        assert!(SemVer::parse("v2.0.0-rc.1") < SemVer::parse("v2.0.0"));
        assert!(SemVer::parse("v2.0.0-alpha.1") < SemVer::parse("v2.0.0"));
    }

    #[test]
    fn compare_prerelease_same_version() {
        let rc1 = SemVer::parse("v2.0.0-rc.1").unwrap();
        let rc2 = SemVer::parse("v2.0.0-rc.2").unwrap();
        assert!(rc1 < rc2);
    }

    // ── compare_versions ───────────────────────────────────────────────────

    #[test]
    fn compare_versions_up_to_date() {
        let info = compare_versions("2.0.0", "2.0.0");
        assert_eq!(info.status, UpdateStatus::UpToDate);
    }

    #[test]
    fn compare_versions_update_available() {
        let info = compare_versions("1.3.0", "2.0.0");
        assert_eq!(info.status, UpdateStatus::UpdateAvailable);
    }

    #[test]
    fn compare_versions_rc_vs_stable() {
        let info = compare_versions("2.0.0-rc.1", "2.0.0");
        // rc.1 < stable, so update is available
        assert_eq!(info.status, UpdateStatus::UpdateAvailable);
    }

    #[test]
    fn compare_versions_current_newer_than_latest() {
        let info = compare_versions("2.1.0", "2.0.0");
        assert_eq!(info.status, UpdateStatus::CurrentIsNewer);
    }

    #[test]
    fn compare_versions_prerelease_newer_line() {
        let info = compare_versions("2.1.0-rc.1", "1.3.0");
        assert_eq!(info.status, UpdateStatus::CurrentIsPrereleaseNewer);
    }

    #[test]
    fn compare_versions_same_numeric_current_is_pre() {
        let info = compare_versions("2.0.0-rc.1", "2.0.0");
        assert_eq!(info.status, UpdateStatus::UpdateAvailable);
    }

    #[test]
    fn compare_versions_invalid_tag_assumes_update() {
        let info = compare_versions("not-a-version", "also-bad");
        assert_eq!(info.status, UpdateStatus::UpdateAvailable);
    }

    #[test]
    fn compare_versions_preserves_strings() {
        let info = compare_versions("2.0.0-rc.1", "2.0.0");
        assert_eq!(info.current, "2.0.0-rc.1");
        assert_eq!(info.latest, "2.0.0");
    }

    // ── extract_tag_name ───────────────────────────────────────────────────

    #[test]
    fn extract_tag_name_normal_json() {
        let json = r#"{"tag_name":"v1.3.0","name":"Zenritme v1.3.0","draft":false}"#;
        assert_eq!(extract_tag_name(json), Some("v1.3.0".to_string()));
    }

    #[test]
    fn extract_tag_name_with_spaces() {
        let json = r#"{
            "id": 12345,
            "tag_name": "v2.0.0-rc.1",
            "draft": false
        }"#;
        assert_eq!(extract_tag_name(json), Some("v2.0.0-rc.1".to_string()));
    }

    #[test]
    fn extract_tag_name_with_escapes() {
        let json = r#"{"tag_name":"v1.0.0-beta\"quoted\"","draft":false}"#;
        assert_eq!(
            extract_tag_name(json),
            Some("v1.0.0-beta\"quoted\"".to_string())
        );
    }

    #[test]
    fn extract_tag_name_missing_key() {
        let json = r#"{"name":"Zenritme","draft":false}"#;
        assert_eq!(extract_tag_name(json), None);
    }

    #[test]
    fn extract_tag_name_empty_json() {
        assert_eq!(extract_tag_name("{}"), None);
    }

    // ── normalize_version ───────────────────────────────────────────────────

    #[test]
    fn normalize_without_v() {
        assert_eq!(normalize_version("1.3.0"), "v1.3.0");
    }

    #[test]
    fn normalize_with_v() {
        assert_eq!(normalize_version("v1.3.0"), "v1.3.0");
    }

    #[test]
    fn normalize_prerelease() {
        assert_eq!(normalize_version("v2.0.0-rc.1"), "v2.0.0-rc.1");
    }

    #[test]
    fn normalize_without_v_prerelease() {
        assert_eq!(normalize_version("2.0.0-rc.1"), "v2.0.0-rc.1");
    }

    #[test]
    fn normalize_empty_does_not_panic() {
        assert_eq!(normalize_version(""), "v");
    }

    #[test]
    fn normalize_whitespace_trimmed() {
        assert_eq!(normalize_version("  v1.3.0  "), "v1.3.0");
    }

    // ── interpret_curl_exit ─────────────────────────────────────────────────

    #[test]
    fn curl_exit_dns() {
        assert_eq!(interpret_curl_exit(6), "DNS resolution failed");
    }

    #[test]
    fn curl_exit_connection_refused() {
        assert_eq!(interpret_curl_exit(7), "connection refused");
    }

    #[test]
    fn curl_exit_timeout() {
        assert_eq!(interpret_curl_exit(28), "network request timed out");
    }

    #[test]
    fn curl_exit_ssl() {
        assert_eq!(interpret_curl_exit(35), "SSL/TLS handshake failed");
    }

    #[test]
    fn curl_exit_unknown() {
        assert_eq!(interpret_curl_exit(99), "network request failed");
    }

    // ── interpret_http_status ───────────────────────────────────────────────

    #[test]
    fn http_404() {
        assert_eq!(
            interpret_http_status(404),
            "no latest GitHub release found for oxyzenQ/zenritme"
        );
    }

    #[test]
    fn http_403() {
        assert_eq!(
            interpret_http_status(403),
            "GitHub API request was rate-limited or forbidden"
        );
    }

    #[test]
    fn http_500() {
        assert_eq!(
            interpret_http_status(500),
            "GitHub API returned an unexpected error"
        );
    }
}
