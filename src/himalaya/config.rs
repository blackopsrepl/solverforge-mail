use std::path::PathBuf;
use std::sync::OnceLock;

/// Cached path to the himalaya binary.
static HIMALAYA_BIN: OnceLock<PathBuf> = OnceLock::new();

/// Locate the himalaya binary, checking several well-known paths.
pub fn himalaya_bin() -> &'static PathBuf {
    HIMALAYA_BIN.get_or_init(|| {
        // 1. solverforge-himalaya on PATH
        if let Ok(path) = which("solverforge-himalaya") {
            return path;
        }
        // 2. himalaya on PATH
        if let Ok(path) = which("himalaya") {
            return path;
        }
        // 3. Known build location
        let build_path = PathBuf::from("/opt/himalaya/target/release/himalaya");
        if build_path.exists() {
            return build_path;
        }
        // Fallback — hope it's on PATH at runtime
        PathBuf::from("himalaya")
    })
}

/// Simple `which` implementation — search PATH for an executable.
fn which(name: &str) -> Result<PathBuf, ()> {
    let path_var = std::env::var("PATH").map_err(|_| ())?;
    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(())
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
