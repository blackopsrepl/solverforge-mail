use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Locate the himalaya binary, checking installed SolverForge paths first.
pub fn himalaya_bin() -> Result<PathBuf> {
    resolve_himalaya_bin(
        dirs::home_dir().as_deref(),
        std::env::var_os("PATH").as_deref(),
    )
}

/// Return the configured Himalaya config hint for display and diagnostics.
///
/// This is not used as a runtime precondition; the Himalaya CLI remains the
/// source of truth and may also resolve its config via `HIMALAYA_CONFIG`.
pub fn configured_config_path() -> PathBuf {
    resolve_config_path(
        std::env::var_os("HIMALAYA_CONFIG").as_deref(),
        dirs::config_dir().as_deref(),
        dirs::home_dir().as_deref(),
    )
}

fn resolve_himalaya_bin(home_dir: Option<&Path>, path_var: Option<&OsStr>) -> Result<PathBuf> {
    for candidate in install_candidates(home_dir) {
        if is_executable_file(&candidate) {
            return Ok(candidate);
        }
    }

    if let Some(path) = which_in_path("solverforge-himalaya", path_var) {
        return Ok(path);
    }
    if let Some(path) = which_in_path("himalaya", path_var) {
        return Ok(path);
    }

    let build_path = PathBuf::from("/opt/himalaya/target/release/himalaya");
    if is_executable_file(&build_path) {
        return Ok(build_path);
    }

    bail!(
        "Himalaya backend not found. Checked ~/.local/share/solverforge/bin/solverforge-himalaya, ~/.local/bin/solverforge-himalaya, PATH for solverforge-himalaya/himalaya, and /opt/himalaya/target/release/himalaya."
    )
}

fn resolve_config_path(
    explicit: Option<&OsStr>,
    config_dir: Option<&Path>,
    home_dir: Option<&Path>,
) -> PathBuf {
    if let Some(path) = explicit {
        return PathBuf::from(path);
    }
    if let Some(dir) = config_dir {
        return dir.join("himalaya").join("config.toml");
    }
    if let Some(home) = home_dir {
        return home.join(".config").join("himalaya").join("config.toml");
    }
    PathBuf::from(".config")
        .join("himalaya")
        .join("config.toml")
}

fn install_candidates(home_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(home_dir) = home_dir {
        candidates.push(
            home_dir
                .join(".local")
                .join("share")
                .join("solverforge")
                .join("bin")
                .join("solverforge-himalaya"),
        );
        candidates.push(
            home_dir
                .join(".local")
                .join("bin")
                .join("solverforge-himalaya"),
        );
    }
    candidates
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

// Simple `which` implementation — search PATH for an executable.
fn which_in_path(name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    let path_var = path_var?;
    for dir in std::env::split_paths(path_var) {
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Build the global args prefix (output format only).
/// Account flag (-a) must go on the subcommand, not here.
pub fn global_args() -> Vec<String> {
    vec!["-o".to_string(), "json".to_string()]
}

/// Build account args to append after the subcommand verb.
pub fn account_args(account: Option<&str>) -> Vec<String> {
    match account {
        Some(acct) => vec!["-a".to_string(), acct.to_string()],
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("solverforge-mail-{prefix}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prefers_solverforge_install_path_over_path_entries() {
        let home = temp_dir("home");
        let install = home
            .join(".local")
            .join("share")
            .join("solverforge")
            .join("bin")
            .join("solverforge-himalaya");
        fs::create_dir_all(install.parent().unwrap()).unwrap();
        fs::write(&install, "binary").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&install, fs::Permissions::from_mode(0o755)).unwrap();

        let path_dir = temp_dir("path");
        let path_bin = path_dir.join("himalaya");
        fs::write(&path_bin, "binary").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&path_bin, fs::Permissions::from_mode(0o755)).unwrap();

        let resolved = resolve_himalaya_bin(Some(&home), Some(path_dir.as_os_str())).unwrap();
        assert_eq!(resolved, install);
    }

    #[test]
    fn falls_back_to_path_lookup() {
        let path_dir = temp_dir("path-only");
        let path_bin = path_dir.join("solverforge-himalaya");
        fs::write(&path_bin, "binary").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&path_bin, fs::Permissions::from_mode(0o755)).unwrap();

        let resolved = resolve_himalaya_bin(None, Some(path_dir.as_os_str())).unwrap();
        assert_eq!(resolved, path_bin);
    }

    #[test]
    fn skips_non_executable_installed_backend_when_path_has_working_binary() {
        let home = temp_dir("home-nonexec");
        let install = home
            .join(".local")
            .join("share")
            .join("solverforge")
            .join("bin")
            .join("solverforge-himalaya");
        fs::create_dir_all(install.parent().unwrap()).unwrap();
        fs::write(&install, "binary").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&install, fs::Permissions::from_mode(0o644)).unwrap();

        let path_dir = temp_dir("path-exec");
        let path_bin = path_dir.join("solverforge-himalaya");
        fs::write(&path_bin, "binary").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&path_bin, fs::Permissions::from_mode(0o755)).unwrap();

        let resolved = resolve_himalaya_bin(Some(&home), Some(path_dir.as_os_str())).unwrap();
        assert_eq!(resolved, path_bin);
    }

    #[test]
    fn config_path_uses_config_dir_when_available() {
        let config_dir = PathBuf::from("/tmp/solverforge-config");
        let resolved = resolve_config_path(None, Some(&config_dir), None);
        assert_eq!(resolved, config_dir.join("himalaya").join("config.toml"));
    }

    #[test]
    fn config_path_prefers_himalaya_config_env() {
        let resolved = resolve_config_path(
            Some(OsStr::new("/tmp/custom-himalaya.toml")),
            Some(Path::new("/tmp/ignored")),
            None,
        );
        assert_eq!(resolved, PathBuf::from("/tmp/custom-himalaya.toml"));
    }
}
