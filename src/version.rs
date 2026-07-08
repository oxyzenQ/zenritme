// SPDX-License-Identifier: GPL-3.0-only
//
// Zenritme
// Copyright (C) 2026 rezky_nightky (oxyzenQ)

/// Build and project metadata for the version report.
///
/// Dynamic build target label: detects arch + libc env at compile time.
/// Returns e.g. "linux-amd64-gnu" (glibc, dynamic) or "linux-amd64-musl"
/// (static) for x86_64 Linux builds.
fn build_label() -> &'static str {
    if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "musl"
    )) {
        "linux-amd64-musl"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        "linux-amd64-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-amd64"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "aarch64",
        target_env = "musl"
    )) {
        "linux-aarch64-musl"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "aarch64",
        target_env = "gnu"
    )) {
        "linux-aarch64-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-aarch64"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "unknown"
    }
}

pub fn version_report() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    let commit = option_env!("ZENRITME_GIT_SHORT").unwrap_or("unknown");
    let target = build_label();

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

    #[test]
    fn build_label_detects_libc_variant() {
        let label = build_label();
        // On x86_64 Linux, label must include -gnu or -musl suffix
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            assert!(
                label.ends_with("-gnu") || label.ends_with("-musl"),
                "build_label must include libc variant (-gnu/-musl), got: {label}"
            );
        }
    }
}
