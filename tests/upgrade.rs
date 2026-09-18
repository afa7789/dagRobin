//! Integration + function-level tests for the upgrade module (src/upgrade.rs).
//!
//! Process-level tests drive the real binary via `env!("CARGO_BIN_EXE_dagRobin")`
//! (mirroring the helper style of tests/cli_archive.rs, minus `--db` — the
//! upgrade subcommand is handled before the database is ever opened).
//! Function-level tests call the public `dagrobin::upgrade` API directly.
//!
//! Hermeticity: every network-capable path is pinned with
//! `DAGROBIN_UPGRADE_LATEST_URL` (dead localhost port) or
//! `DAGROBIN_UPGRADE_EXE` (ephemeral channel), so no test ever touches the
//! real network, and no test ever runs `npm`.

use dagrobin::upgrade::{
    artifact_name, compare_versions, detect_channel, hardlinked_siblings, replace_executable,
    InstallChannel,
};
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Mirror of the tests/cli_archive.rs helper, minus `--db`: the upgrade
/// subcommand returns before the database is ever opened.
fn dagrobin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dagRobin"))
}

#[test]
fn compare_versions_numeric() {
    assert_eq!(compare_versions("0.1.0", "0.1.1").unwrap(), Ordering::Less);
    assert_eq!(compare_versions("0.1.1", "0.1.1").unwrap(), Ordering::Equal);
    assert_eq!(
        compare_versions("0.1.10", "0.1.2").unwrap(),
        Ordering::Greater
    );
    // Numeric, not lexicographic: 0.10.0 > 0.2.0 therefore 0.2.0 < 0.10.0.
    assert_eq!(compare_versions("0.2.0", "0.10.0").unwrap(), Ordering::Less);
}

#[test]
fn compare_versions_v_prefix() {
    assert_eq!(
        compare_versions("v0.1.1", "0.1.1").unwrap(),
        Ordering::Equal
    );
    assert_eq!(compare_versions("0.1.1", "v0.1.2").unwrap(), Ordering::Less);
}

#[test]
fn compare_versions_suffix_ignored() {
    assert_eq!(
        compare_versions("0.1.1-rc.1", "0.1.1").unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        compare_versions("0.1.1+build5", "0.1.1").unwrap(),
        Ordering::Equal
    );
}

#[test]
fn compare_versions_invalid() {
    for (a, b) in [("abc", "0.1.1"), ("1.2", "0.1.1"), ("0.1.1", "1.2.3.4")] {
        let err = compare_versions(a, b).unwrap_err().to_string();
        assert!(err.contains("Invalid version"), "got: {err}");
    }
}

#[test]
fn artifact_name_resolves_all_targets() {
    for (os, arch, expected) in [
        ("macos", "x86_64", "dagRobin-macos-amd64"),
        ("macos", "aarch64", "dagRobin-macos-arm64"),
        ("linux", "x86_64", "dagRobin-linux-amd64"),
        ("linux", "aarch64", "dagRobin-linux-arm64"),
    ] {
        assert_eq!(artifact_name(os, arch).unwrap(), expected);
    }
}

#[test]
fn artifact_name_unsupported_platform() {
    let err = artifact_name("windows", "x86_64").unwrap_err().to_string();
    assert!(err.contains("no prebuilt binary"), "got: {err}");
    assert!(
        err.contains("cargo install --git https://github.com/afa7789/dagRobin"),
        "got: {err}"
    );
}

#[test]
fn detect_channel_cargo_and_raw() {
    assert_eq!(
        detect_channel(&PathBuf::from("/Users/x/.cargo/bin/dagRobin")),
        InstallChannel::Standalone
    );
    assert_eq!(
        detect_channel(&PathBuf::from("/usr/local/bin/dagRobin")),
        InstallChannel::Standalone
    );
}

#[test]
fn detect_channel_npm() {
    assert_eq!(
        detect_channel(&PathBuf::from(
            "/opt/homebrew/lib/node_modules/dagrobin/bin/dagRobin"
        )),
        InstallChannel::Npm
    );
}

#[test]
fn detect_channel_ephemeral() {
    assert_eq!(
        detect_channel(&PathBuf::from(
            "/Users/x/.npm/_npx/abc123/node_modules/dagrobin/bin/dagRobin"
        )),
        InstallChannel::Ephemeral
    );
    assert_eq!(
        detect_channel(&PathBuf::from(
            "/Users/x/.bun/install/cache/dagrobin@0.1.1/node_modules/dagrobin/bin/dagRobin"
        )),
        InstallChannel::Ephemeral
    );
}

#[cfg(unix)]
#[test]
fn replace_executable_replaces_and_relinks_siblings() {
    use std::os::unix::fs::MetadataExt;

    let dir = TempDir::new().unwrap();
    let old = dir.path().join("old");
    let sibling = dir.path().join("sibling");
    let new = dir.path().join("new");
    fs::write(&old, b"old-binary").unwrap();
    fs::hard_link(&old, &sibling).unwrap();
    fs::write(&new, b"new-binary").unwrap();

    replace_executable(&old, &new, "staging").unwrap();

    assert_eq!(fs::read(&old).unwrap(), b"new-binary");
    assert_eq!(fs::read(&sibling).unwrap(), b"new-binary");
    let old_meta = fs::metadata(&old).unwrap();
    let sibling_meta = fs::metadata(&sibling).unwrap();
    assert_eq!(old_meta.dev(), sibling_meta.dev());
    assert_eq!(old_meta.ino(), sibling_meta.ino());
    assert!(!dir.path().join("staging").exists());
    // The source of the staged copy is never touched.
    assert_eq!(fs::read(&new).unwrap(), b"new-binary");
}

#[cfg(unix)]
#[test]
fn hardlinked_siblings_finds_only_links() {
    use std::os::unix::fs::MetadataExt;

    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target");
    fs::write(&target, b"x").unwrap();
    for name in ["a", "b", "staging"] {
        fs::hard_link(&target, dir.path().join(name)).unwrap();
    }
    fs::write(dir.path().join("c"), b"unrelated").unwrap();

    let meta = fs::metadata(&target).unwrap();
    let found =
        hardlinked_siblings(dir.path(), "target", "staging", meta.dev(), meta.ino()).unwrap();
    let mut names: Vec<&str> = found
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .collect();
    names.sort_unstable();
    // Exactly the hardlinks `a` and `b` — never `target`, never `c`, and the
    // staging-named hardlink is excluded.
    assert_eq!(names, vec!["a", "b"], "found: {found:?}");
}

#[test]
fn upgrade_check_offline_reports_clean_error() {
    let out = dagrobin()
        .env("DAGROBIN_UPGRADE_LATEST_URL", "http://127.0.0.1:1/latest")
        .args(["upgrade", "--check"])
        .output()
        .expect("run dagRobin upgrade --check");
    assert_eq!(out.status.code(), Some(1));
    // The failure is reported on stderr, never as a panic/`Error:` on stdout.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Could not reach the latest-release API"),
        "stderr: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("Error:"), "stdout: {stdout}");
}

#[test]
fn upgrade_ephemeral_is_noop() {
    // Path shaped like an npx-managed install, created on disk so that
    // canonicalization inside resolve_exe_path() cannot fail.
    let dir = TempDir::new().unwrap();
    let dummy = dir
        .path()
        .join("_npx/abc/node_modules/dagrobin/bin/dagRobin");
    fs::create_dir_all(dummy.parent().unwrap()).unwrap();
    fs::write(&dummy, b"dummy").unwrap();

    let out = dagrobin()
        .env("DAGROBIN_UPGRADE_EXE", &dummy)
        .arg("upgrade")
        .output()
        .expect("run dagRobin upgrade");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no-op"), "stdout: {stdout}");
    // Proof the npm-delegation branch was never entered.
    assert!(
        !stdout.contains("npm-managed install detected"),
        "stdout: {stdout}"
    );
}

#[test]
fn upgrade_help_lists_flags() {
    let out = dagrobin()
        .args(["upgrade", "--help"])
        .output()
        .expect("run dagRobin upgrade --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--check"), "stdout: {stdout}");
    assert!(stdout.contains("--force"), "stdout: {stdout}");
    assert!(stdout.contains("Self-update"), "stdout: {stdout}");
}
