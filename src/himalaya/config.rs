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

/// Build the base command prefix with optional account flag.
pub fn base_args(account: Option<&str>) -> Vec<String> {
    let mut args = vec!["-o".to_string(), "json".to_string()];
    if let Some(acct) = account {
        args.push("-a".to_string());
        args.push(acct.to_string());
    }
    args
}
