//! Upgrade logic: install-channel detection, latest-release lookup, download
//! with SHA256 verification, and atomic replacement of the running binary.

use std::{
    cmp::Ordering,
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;

use crate::error::{DagRobinError, Result};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

const REPO: &str = "afa7789/dagRobin";
const DEFAULT_LATEST_URL: &str = "https://api.github.com/repos/afa7789/dagRobin/releases/latest";
const DOWNLOAD_BASE: &str = "https://github.com/afa7789/dagRobin/releases/download";

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstallChannel {
    Standalone,
    Npm,
    Ephemeral,
}

pub fn run(check: bool, force: bool) -> Result<()> {
    let exe_path = resolve_exe_path()?;

    match detect_channel(&exe_path) {
        InstallChannel::Ephemeral => {
            println!(
                "dagRobin upgrade is a no-op for ephemeral npx/bunx runs; install globally with: npm install -g dagrobin@latest"
            );
            return Ok(());
        }
        InstallChannel::Npm => return delegate_to_npm(),
        InstallChannel::Standalone => {}
    }

    let artifact = artifact_name(env::consts::OS, env::consts::ARCH)?;
    let tag = fetch_latest_tag()?;
    let current = env!("CARGO_PKG_VERSION");
    let ordering = compare_versions(current, &tag)?;

    if ordering == Ordering::Greater {
        println!(
            "Installed version {} is newer than the latest release {}",
            display_current(),
            tag
        );
        return Ok(());
    }

    if check {
        if ordering == Ordering::Less {
            println!("Update available: {} -> {}", display_current(), tag);
            process::exit(1);
        }
        println!("Already up to date ({})", display_current());
        return Ok(());
    }

    println!("Current version: {}", display_current());
    println!("Latest version: {}", tag);

    if ordering == Ordering::Equal && !force {
        println!("Already up to date ({})", display_current());
        return Ok(());
    }

    let tmp = env::temp_dir().join(format!(
        "dagrobin-upgrade-{}-{}",
        process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let result = (|| -> Result<()> {
        fs::create_dir_all(&tmp).map_err(|io| DagRobinError::UpgradeFailed {
            message: format!(
                "Failed to create temporary directory {}: {}",
                tmp.display(),
                io
            ),
        })?;
        println!("Downloading {artifact}.tar.gz...");
        let bin = download_and_verify(&artifact, &tag, &tmp)?;
        replace_executable(
            &exe_path,
            &bin,
            &format!(".dagRobin.upgrade.{}", process::id()),
        )?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&tmp);
    result?;

    println!("Upgraded to {}", tag);
    Ok(())
}

fn resolve_exe_path() -> Result<PathBuf> {
    let exe = match env::var_os("DAGROBIN_UPGRADE_EXE") {
        Some(path) => PathBuf::from(path),
        None => env::current_exe().map_err(|io| DagRobinError::UpgradeFailed {
            message: format!("Cannot determine the running executable: {}", io),
        })?,
    };
    fs::canonicalize(&exe).map_err(|io| DagRobinError::UpgradeFailed {
        message: format!("Cannot determine the running executable: {}", io),
    })
}

fn delegate_to_npm() -> Result<()> {
    println!("npm-managed install detected — running: npm install -g dagrobin@latest");
    match Command::new("npm")
        .args(["install", "-g", "dagrobin@latest"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(DagRobinError::UpgradeFailed {
            message: "npm not found on PATH. Run this manually: npm install -g dagrobin@latest"
                .into(),
        }),
        Err(e) => Err(DagRobinError::UpgradeFailed {
            message: format!("Failed to run npm: {}", e),
        }),
    }
}

fn fetch_latest_tag() -> Result<String> {
    let url =
        env::var("DAGROBIN_UPGRADE_LATEST_URL").unwrap_or_else(|_| DEFAULT_LATEST_URL.to_string());
    let body = curl_stdout(&["-f", "-s", "-S", "-L", "--max-time", "30", &url]).map_err(|e| {
        DagRobinError::UpgradeFailed {
            message: format!(
                "Could not reach the latest-release API: {} — check your network connection and retry.",
                e
            ),
        }
    })?;
    let release: LatestRelease =
        serde_json::from_str(&body).map_err(|e| DagRobinError::UpgradeFailed {
            message: format!("Invalid response from GitHub API: {}", e),
        })?;
    Ok(release.tag_name)
}

fn curl_stdout(args: &[&str]) -> Result<String> {
    let output = Command::new("curl")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                DagRobinError::UpgradeFailed {
                    message: format!(
                        "curl not found on PATH. Install curl or update manually: https://github.com/{}/releases",
                        REPO
                    ),
                }
            } else {
                DagRobinError::UpgradeFailed {
                    message: format!("curl failed (exit {}): {}", -1, e),
                }
            }
        })?;
    if !output.status.success() {
        let detail = last_nonempty_stderr_line(&output.stderr);
        let message = if detail.is_empty() {
            format!("curl failed (exit {})", output.status.code().unwrap_or(-1))
        } else {
            format!(
                "curl failed (exit {}): {}",
                output.status.code().unwrap_or(-1),
                detail
            )
        };
        return Err(DagRobinError::UpgradeFailed { message });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn curl_to_file(args: &[&str], out: &Path) -> Result<()> {
    let out_str = out.to_string_lossy();
    let mut full_args = args.to_vec();
    full_args.push("-o");
    full_args.push(out_str.as_ref());
    curl_stdout(&full_args)?;
    let meta = fs::metadata(out).map_err(|io| DagRobinError::UpgradeFailed {
        message: format!(
            "Failed to inspect downloaded file {}: {}",
            out.display(),
            io
        ),
    })?;
    if meta.len() == 0 {
        return Err(DagRobinError::UpgradeFailed {
            message: "Download produced an empty file".into(),
        });
    }
    Ok(())
}

fn download_and_verify(artifact: &str, tag: &str, tmp: &Path) -> Result<PathBuf> {
    let base = format!("{DOWNLOAD_BASE}/{tag}");
    let tar_name = format!("{artifact}.tar.gz");
    let tar_url = format!("{base}/{tar_name}");
    let sha_url = format!("{base}/{tar_name}.sha256");
    let tar_path = tmp.join(&tar_name);
    let sha_path = tmp.join(format!("{tar_name}.sha256"));

    let download = |url: &str, out: &Path| -> Result<()> {
        curl_to_file(&["-f", "-s", "-S", "-L", "--max-time", "120", url], out).map_err(|e| {
            let detail = e.to_string();
            let hint = if detail.contains("404") {
                " (the tag may not have this asset)"
            } else {
                ""
            };
            DagRobinError::UpgradeFailed {
                message: format!(
                    "Download failed for {}: {} — check your network connection and retry.{}",
                    url, detail, hint
                ),
            }
        })
    };
    download(&tar_url, &tar_path)?;
    download(&sha_url, &sha_path)?;

    let sha_text = fs::read_to_string(&sha_path).map_err(|io| DagRobinError::UpgradeFailed {
        message: format!(
            "Failed to read checksum file {}: {}",
            sha_path.display(),
            io
        ),
    })?;
    let expected = match sha_text.split_whitespace().next() {
        Some(token) if token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()) => {
            token.to_string().to_lowercase()
        }
        _ => {
            return Err(DagRobinError::UpgradeFailed {
                message: format!(
                    "Invalid checksum file {}: expected a hex SHA256",
                    sha_path.display()
                ),
            });
        }
    };

    println!("Verifying SHA256...");
    let actual = sha256_hex(&tar_path)?;
    if actual != expected {
        return Err(DagRobinError::UpgradeFailed {
            message: format!(
                "Checksum mismatch for {}: expected {}, got {}",
                tar_name, expected, actual
            ),
        });
    }

    let tar_str = tar_path.to_string_lossy();
    let tmp_str = tmp.to_string_lossy();
    let extract = Command::new("tar")
        .args(["-xzf", tar_str.as_ref(), "-C", tmp_str.as_ref()])
        .output()
        .map_err(|io| DagRobinError::UpgradeFailed {
            message: if io.kind() == ErrorKind::NotFound {
                "tar not found on PATH: cannot extract the downloaded archive".into()
            } else {
                format!("Failed to run tar: {}", io)
            },
        })?;
    if !extract.status.success() {
        return Err(DagRobinError::UpgradeFailed {
            message: format!(
                "Failed to extract {}: {}",
                tar_path.display(),
                last_nonempty_stderr_line(&extract.stderr)
            ),
        });
    }

    let bin = tmp.join("dagRobin");
    if bin.exists() && bin.is_file() {
        Ok(bin)
    } else {
        Err(DagRobinError::UpgradeFailed {
            message: "Extracted archive does not contain a dagRobin binary".into(),
        })
    }
}

fn sha256_hex(path: &Path) -> Result<String> {
    let shasum_flags = ["-a", "256"];
    for (cmd, flags) in [("shasum", &shasum_flags[..]), ("sha256sum", &[][..])] {
        let Ok(out) = Command::new(cmd).args(flags).arg(path).output() else {
            continue;
        };
        if out.status.success() {
            if let Some(token) = String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .next()
            {
                return Ok(token.trim().to_lowercase());
            }
        }
    }
    Err(DagRobinError::UpgradeFailed {
        message: "Neither shasum nor sha256sum is available to verify the downloaded checksum"
            .into(),
    })
}

pub fn compare_versions(a: &str, b: &str) -> Result<Ordering> {
    Ok(parse_version(a)?.cmp(&parse_version(b)?))
}

fn parse_version(s: &str) -> Result<(u64, u64, u64)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let parts: Vec<&str> = core.split('.').collect();
    let invalid = || DagRobinError::UpgradeFailed {
        message: format!(
            "Invalid version '{}': expected major.minor.patch (numeric)",
            s
        ),
    };
    if parts.len() != 3 {
        return Err(invalid());
    }
    let major = parts[0].parse::<u64>().map_err(|_| invalid())?;
    let minor = parts[1].parse::<u64>().map_err(|_| invalid())?;
    let patch = parts[2].parse::<u64>().map_err(|_| invalid())?;
    Ok((major, minor, patch))
}

pub fn artifact_name(os: &str, arch: &str) -> Result<String> {
    let artifact = match (os, arch) {
        ("macos", "x86_64") => "dagRobin-macos-amd64",
        ("macos", "aarch64") => "dagRobin-macos-arm64",
        ("linux", "x86_64") => "dagRobin-linux-amd64",
        ("linux", "aarch64") => "dagRobin-linux-arm64",
        _ => {
            return Err(DagRobinError::UpgradeFailed {
                message: format!(
                    "dagRobin has no prebuilt binary for {}-{}. Build from source instead: cargo install --git https://github.com/{}",
                    os, arch, REPO
                ),
            });
        }
    };
    Ok(artifact.to_string())
}

pub fn detect_channel(exe_path: &Path) -> InstallChannel {
    let components: Vec<&str> = exe_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    if !components.contains(&"node_modules") {
        return InstallChannel::Standalone;
    }
    if components.iter().any(|c| *c == "_npx" || *c == ".bun") {
        return InstallChannel::Ephemeral;
    }
    InstallChannel::Npm
}

pub fn replace_executable(exe_path: &Path, new_binary: &Path, staging_name: &str) -> Result<()> {
    let dir = match exe_path.parent() {
        Some(dir) => dir,
        None => {
            return Err(DagRobinError::UpgradeFailed {
                message: "Current executable has no parent directory".into(),
            });
        }
    };
    let staging = dir.join(staging_name);

    let result = (|| -> Result<()> {
        fs::copy(new_binary, &staging).map_err(|io| DagRobinError::UpgradeFailed {
            message: format!(
                "Failed to stage new binary at {}: {}",
                staging.display(),
                io
            ),
        })?;
        #[cfg(unix)]
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o755)).map_err(|io| {
            DagRobinError::UpgradeFailed {
                message: format!(
                    "Failed to stage new binary at {}: {}",
                    staging.display(),
                    io
                ),
            }
        })?;
        let old_ident = metadata_identity(exe_path)?;
        fs::rename(&staging, exe_path).map_err(|io| DagRobinError::UpgradeFailed {
            message: format!("Failed to replace {}: {}", exe_path.display(), io),
        })?;
        if let Some((dev, ino)) = old_ident {
            let exe_name = exe_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            for sibling in hardlinked_siblings(dir, exe_name, staging_name, dev, ino)? {
                fs::remove_file(&sibling).map_err(|io| DagRobinError::UpgradeFailed {
                    message: format!("Failed to re-link sibling {}: {}", sibling.display(), io),
                })?;
                fs::hard_link(exe_path, &sibling).map_err(|io| DagRobinError::UpgradeFailed {
                    message: format!("Failed to re-link sibling {}: {}", sibling.display(), io),
                })?;
            }
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result
}

fn metadata_identity(path: &Path) -> Result<Option<(u64, u64)>> {
    #[cfg(not(unix))]
    {
        let _ = path;
        return Ok(None);
    }
    #[cfg(unix)]
    {
        match fs::metadata(path) {
            Ok(meta) => Ok(Some((meta.dev(), meta.ino()))),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DagRobinError::UpgradeFailed {
                message: format!("Failed to read metadata for {}: {}", path.display(), e),
            }),
        }
    }
}

pub fn hardlinked_siblings(
    dir: &Path,
    exe_name: &str,
    staging_name: &str,
    old_dev: u64,
    old_ino: u64,
) -> Result<Vec<PathBuf>> {
    #[cfg(not(unix))]
    {
        let _ = (dir, exe_name, staging_name, old_dev, old_ino);
        return Ok(Vec::new());
    }
    #[cfg(unix)]
    {
        let mut siblings = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if name.to_str() == Some(exe_name) || name.to_str() == Some(staging_name) {
                continue;
            }
            if !entry.file_type()?.is_file() {
                continue;
            }
            let Ok(meta) = fs::metadata(entry.path()) else {
                continue;
            };
            if meta.dev() == old_dev && meta.ino() == old_ino {
                siblings.push(entry.path());
            }
        }
        Ok(siblings)
    }
}

fn last_nonempty_stderr_line(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

fn display_current() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard(Vec<(&'static str, Option<String>)>);

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> EnvGuard {
            let prev = env::var(key).ok();
            env::set_var(key, value);
            EnvGuard(vec![(key, prev)])
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, prev) in &self.0 {
                if let Some(value) = prev {
                    env::set_var(key, value);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }

    #[test]
    fn parse_version_accepts_valid() {
        assert_eq!(parse_version("0.1.1").unwrap(), (0, 1, 1));
        assert_eq!(parse_version("v1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(parse_version("10.20.30-pre.1").unwrap(), (10, 20, 30));
        assert_eq!(parse_version("1.2.3+meta").unwrap(), (1, 2, 3));
    }

    #[test]
    fn parse_version_rejects_malformed() {
        for bad in ["1.2", "1.2.3.4", "abc", "", "v", "v1.2", "1..2"] {
            let err = parse_version(bad).unwrap_err().to_string();
            assert!(
                err.contains("Invalid version"),
                "expected Invalid version for {bad:?}, got {err}"
            );
        }
        assert_eq!(
            parse_version("1.2").unwrap_err().to_string(),
            "Invalid version '1.2': expected major.minor.patch (numeric)"
        );
        assert_eq!(
            parse_version("v1.2").unwrap_err().to_string(),
            "Invalid version '1.2': expected major.minor.patch (numeric)"
        );
    }

    #[test]
    fn curl_stdout_and_curl_to_file_with_file_url() {
        let dir = tempfile::tempdir().unwrap();
        let payload = b"{\"tag_name\":\"v9.9.9\"}";
        let f = dir.path().join("latest.json");
        fs::write(&f, payload).unwrap();
        let url = format!("file://{}", f.display());

        let body = curl_stdout(&["-f", "-s", "-S", "-L", "--max-time", "30", &url]).unwrap();
        assert_eq!(body, "{\"tag_name\":\"v9.9.9\"}");

        let out = dir.path().join("out.txt");
        curl_to_file(&["-f", "-s", "-S", "-L", &url], &out).unwrap();
        assert_eq!(fs::read(&out).unwrap(), payload);
    }

    #[test]
    fn curl_failure_is_wrapped() {
        let err = curl_stdout(&[
            "-f",
            "-s",
            "-S",
            "-L",
            "--max-time",
            "5",
            "file:///nonexistent/nope.json",
        ])
        .unwrap_err()
        .to_string();
        assert!(err.starts_with("curl failed (exit "), "got: {err}");
    }

    #[test]
    fn curl_to_file_rejects_empty_download() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("empty.txt"), b"").unwrap();
        let url = format!("file://{}", dir.path().join("empty.txt").display());
        let out = dir.path().join("out.tgz");
        let err = curl_to_file(&["-f", "-s", "-S", "-L", &url], &out)
            .unwrap_err()
            .to_string();
        assert_eq!(err, "Download produced an empty file");
    }

    #[test]
    fn fetch_latest_tag_uses_env_url() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, b"{\"tag_name\":\"v0.9.9\"}").unwrap();
        let url = format!("file://{}", f.display());
        let _guard = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert_eq!(fetch_latest_tag().unwrap(), "v0.9.9");
    }

    #[test]
    fn fetch_latest_tag_wraps_network_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(
            "DAGROBIN_UPGRADE_LATEST_URL",
            "file:///nonexistent/latest.json",
        );
        let err = fetch_latest_tag().unwrap_err().to_string();
        assert!(
            err.starts_with("Could not reach the latest-release API: "),
            "got: {err}"
        );
        assert!(err.ends_with("— check your network connection and retry."));
    }

    #[test]
    fn fetch_latest_tag_rejects_bad_json() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, b"not json at all").unwrap();
        let url = format!("file://{}", f.display());
        let _guard = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        let err = fetch_latest_tag().unwrap_err().to_string();
        assert!(
            err.starts_with("Invalid response from GitHub API: "),
            "got: {err}"
        );
    }

    #[test]
    fn fetch_latest_tag_rejects_missing_tag_field() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, b"{\"name\":\"no tag here\"}").unwrap();
        let url = format!("file://{}", f.display());
        let _guard = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        let err = fetch_latest_tag().unwrap_err().to_string();
        assert!(
            err.starts_with("Invalid response from GitHub API: "),
            "got: {err}"
        );
    }

    #[test]
    fn sha256_hex_known_vector() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("data");
        fs::write(&f, b"abc").unwrap();
        let hex = sha256_hex(&f).unwrap();
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn resolve_exe_path_prefers_env_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let _guard = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let resolved = resolve_exe_path().unwrap();
        assert_eq!(resolved, fs::canonicalize(&exe).unwrap());
    }

    #[test]
    fn resolve_exe_path_missing_override_errors() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        let _guard = EnvGuard::set("DAGROBIN_UPGRADE_EXE", missing.to_str().unwrap());
        let err = resolve_exe_path().unwrap_err().to_string();
        assert!(
            err.starts_with("Cannot determine the running executable: "),
            "got: {err}"
        );
    }

    #[test]
    fn run_check_up_to_date_is_ok() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let f = dir.path().join("latest.json");
        fs::write(
            &f,
            format!("{{\"tag_name\":\"v{}\"}}", env!("CARGO_PKG_VERSION")),
        )
        .unwrap();
        let url = format!("file://{}", f.display());
        let _g1 = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let _g2 = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert!(run(true, false).is_ok());
    }

    #[test]
    fn run_non_check_up_to_date_is_ok_without_download() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let f = dir.path().join("latest.json");
        fs::write(
            &f,
            format!("{{\"tag_name\":\"v{}\"}}", env!("CARGO_PKG_VERSION")),
        )
        .unwrap();
        let url = format!("file://{}", f.display());
        let _g1 = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let _g2 = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert!(run(false, false).is_ok());
    }

    #[test]
    fn run_rejects_downgrade_without_force() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, "{\"tag_name\":\"v0.0.1\"}").unwrap();
        let url = format!("file://{}", f.display());
        let _g1 = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let _g2 = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert!(run(false, false).is_ok());
        assert_eq!(fs::read_to_string(&exe).unwrap(), "x");
    }

    #[test]
    fn run_rejects_downgrade_even_with_force() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, "{\"tag_name\":\"v0.0.1\"}").unwrap();
        let url = format!("file://{}", f.display());
        let _g1 = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let _g2 = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert!(run(false, true).is_ok());
        assert_eq!(fs::read_to_string(&exe).unwrap(), "x");
    }

    #[test]
    fn run_check_reports_ahead_without_download() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("dagRobin");
        fs::write(&exe, b"x").unwrap();
        let f = dir.path().join("latest.json");
        fs::write(&f, "{\"tag_name\":\"v0.0.1\"}").unwrap();
        let url = format!("file://{}", f.display());
        let _g1 = EnvGuard::set("DAGROBIN_UPGRADE_EXE", exe.to_str().unwrap());
        let _g2 = EnvGuard::set("DAGROBIN_UPGRADE_LATEST_URL", &url);
        assert!(run(true, false).is_ok());
        assert_eq!(fs::read_to_string(&exe).unwrap(), "x");
    }

    #[test]
    fn replace_executable_no_parent_dir() {
        let result = replace_executable(Path::new("/"), Path::new("/tmp/x"), ".staging");
        assert_eq!(
            result.unwrap_err().to_string(),
            "Current executable has no parent directory"
        );
    }

    #[cfg(unix)]
    #[test]
    fn replace_executable_cleans_staging_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let exe_dir = dir.path().join("exe-is-a-dir");
        fs::create_dir(&exe_dir).unwrap();

        let new_dir = tempfile::tempdir().unwrap();
        let new_bin = new_dir.path().join("new");
        fs::write(&new_bin, b"new-binary").unwrap();

        let result = replace_executable(&exe_dir, &new_bin, ".staging");
        assert!(result.is_err());
        assert!(!dir.path().join(".staging").exists());
    }
}
