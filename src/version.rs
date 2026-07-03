// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

/// Build and project metadata for the version report.
///
/// Map Rust's internal arch name to the project's user-facing label.
/// `x86_64` -> `amd64`; all others pass through unchanged.
fn arch_label() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        other => other,
    }
}

pub fn version_report() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    let commit = option_env!("ZENRITME_GIT_SHORT").unwrap_or("unknown");
    let target = format!("{}-{}", std::env::consts::OS, arch_label());

    format!(
        "Version: v{ver}\n\
         Build: {target} ({commit})\n\
         Copyright: (c) 2026 rezky_nightky (oxyzenQ)\n\
         License: GPL-3.0-only\n\
         Source: https://github.com/oxyzenQ/zenritme"
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_report_contains_version_line() {
        let report = version_report();
        assert!(
            report.contains("Version: v"),
            "missing Version line in:\n{report}"
        );
    }

    #[test]
    fn version_report_contains_build_line() {
        let report = version_report();
        assert!(
            report.contains("Build:"),
            "missing Build line in:\n{report}"
        );
    }

    #[test]
    fn version_report_contains_copyright() {
        let report = version_report();
        assert!(
            report.contains("Copyright: (c) 2026 rezky_nightky (oxyzenQ)"),
            "missing Copyright line in:\n{report}"
        );
    }

    #[test]
    fn version_report_contains_license() {
        let report = version_report();
        assert!(
            report.contains("License: GPL-3.0-only"),
            "missing License line in:\n{report}"
        );
    }

    #[test]
    fn version_report_contains_source() {
        let report = version_report();
        assert!(
            report.contains("Source: https://github.com/oxyzenQ/zenritme"),
            "missing Source line in:\n{report}"
        );
    }

    #[test]
    fn version_report_includes_package_version() {
        let report = version_report();
        assert!(
            report.contains(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            "missing package version in:\n{report}"
        );
    }

    #[test]
    fn version_report_has_five_lines() {
        let report = version_report();
        assert_eq!(
            report.lines().count(),
            5,
            "expected exactly 5 lines, got:\n{report}"
        );
    }
}
