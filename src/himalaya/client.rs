use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use super::config::{account_args, global_args, himalaya_bin};
use super::types::*;

// Run a himalaya command and return its stdout.
fn run(args: &[String]) -> Result<String> {
    let bin = himalaya_bin();
    let output = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute: {} {}", bin.display(), args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };
        anyhow::bail!("himalaya error: {}", msg.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/* Run a himalaya command, writing `input` to the child's stdin.
   himalaya's `template send` checks `io::stdin().is_terminal()` and reads
   from stdin when it is not a terminal (i.e. when spawned as a subprocess). */
fn run_with_stdin(args: &[String], input: &str) -> Result<String> {
    let bin = himalaya_bin();
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn: {} {}", bin.display(), args.join(" ")))?;

    // Write the template to stdin, then close it so the child sees EOF.
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .context("failed to write template to himalaya stdin")?;
        // stdin is dropped here, closing the pipe
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for himalaya")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };
        anyhow::bail!("himalaya error: {}", msg.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Build the shell command string for compose/reply/forward operations
// that need to shell out to $EDITOR.
fn editor_command(args: &[String]) -> String {
    let bin = himalaya_bin();
    let mut parts = vec![bin.display().to_string()];
    parts.extend(args.iter().cloned());
    parts.join(" ")
}

// ── Account operations ──────────────────────────────────────────────

/// List all configured accounts.
pub fn list_accounts() -> Result<Vec<Account>> {
    let mut args = global_args();
    args.extend(["account".to_string(), "list".to_string()]);
    let output = run(&args)?;
    let accounts: Vec<Account> =
        serde_json::from_str(&output).context("failed to parse account list")?;
    Ok(accounts)
}

// ── Folder operations ───────────────────────────────────────────────

/// List all folders for the given account.
pub fn list_folders(account: Option<&str>) -> Result<Vec<Folder>> {
    let mut args = global_args();
    args.extend(["folder".to_string(), "list".to_string()]);
    args.extend(account_args(account));
    let output = run(&args)?;
    let folders: Vec<Folder> =
        serde_json::from_str(&output).context("failed to parse folder list")?;
    Ok(folders)
}

// ── Envelope operations ─────────────────────────────────────────────

/// List envelopes in a folder with optional search query.
pub fn list_envelopes(
    account: Option<&str>,
    folder: &str,
    page: usize,
    page_size: usize,
    query: Option<&str>,
) -> Result<Vec<Envelope>> {
    let mut args = global_args();
    args.extend(["envelope".to_string(), "list".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        "-p".to_string(),
        page.to_string(),
        "-s".to_string(),
        page_size.to_string(),
    ]);
    if let Some(q) = query {
        // Split query into words for himalaya's query language
        for word in q.split_whitespace() {
            args.push(word.to_string());
        }
    }
    let output = run(&args)?;
    let envelopes: Vec<Envelope> =
        serde_json::from_str(&output).context("failed to parse envelope list")?;
    Ok(envelopes)
}

/// List envelopes threaded by conversation.
pub fn list_envelopes_threaded(
    account: Option<&str>,
    folder: &str,
    query: Option<&str>,
) -> Result<Vec<Envelope>> {
    let mut args = global_args();
    args.extend(["envelope".to_string(), "thread".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string()]);
    if let Some(q) = query {
        for word in q.split_whitespace() {
            args.push(word.to_string());
        }
    }
    let output = run(&args)?;
    let envelopes: Vec<Envelope> =
        serde_json::from_str(&output).context("failed to parse threaded envelope list")?;
    Ok(envelopes)
}

// ── Message operations ──────────────────────────────────────────────

/// Read a message body (plain text).
pub fn read_message(account: Option<&str>, folder: &str, id: &str) -> Result<String> {
    let mut args = global_args();
    args.extend(["message".to_string(), "read".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    run(&args)
}

/// Read a message in preview mode (don't mark as seen).
pub fn preview_message(account: Option<&str>, folder: &str, id: &str) -> Result<String> {
    let mut args = global_args();
    args.extend(["message".to_string(), "read".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-p".to_string(),
        "-f".to_string(),
        folder.to_string(),
        id.to_string(),
    ]);
    run(&args)
}

/// Delete a message (moves to trash, or flags as deleted if in trash).
pub fn delete_message(account: Option<&str>, folder: &str, id: &str) -> Result<()> {
    let mut args = global_args();
    args.extend(["message".to_string(), "delete".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    run(&args)?;
    Ok(())
}

/// Move a message to a target folder.
pub fn move_message(account: Option<&str>, folder: &str, target: &str, id: &str) -> Result<()> {
    let mut args = global_args();
    args.extend(["message".to_string(), "move".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        target.to_string(),
        id.to_string(),
    ]);
    run(&args)?;
    Ok(())
}

/// Copy a message to a target folder.
pub fn copy_message(account: Option<&str>, folder: &str, target: &str, id: &str) -> Result<()> {
    let mut args = global_args();
    args.extend(["message".to_string(), "copy".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        target.to_string(),
        id.to_string(),
    ]);
    run(&args)?;
    Ok(())
}

// ── Flag operations ─────────────────────────────────────────────────

/// Add a flag to an envelope.
pub fn flag_add(account: Option<&str>, folder: &str, id: &str, flag: &str) -> Result<()> {
    let mut args = global_args();
    args.extend(["flag".to_string(), "add".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        id.to_string(),
        flag.to_string(),
    ]);
    run(&args)?;
    Ok(())
}

/// Remove a flag from an envelope.
pub fn flag_remove(account: Option<&str>, folder: &str, id: &str, flag: &str) -> Result<()> {
    let mut args = global_args();
    args.extend(["flag".to_string(), "remove".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        id.to_string(),
        flag.to_string(),
    ]);
    run(&args)?;
    Ok(())
}

// ── Attachment operations ───────────────────────────────────────────

/// Download attachments for a message.
pub fn download_attachments(account: Option<&str>, folder: &str, id: &str) -> Result<String> {
    let mut args = global_args();
    args.extend(["attachment".to_string(), "download".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    run(&args)
}

// ── Compose operations (shell-out commands) ─────────────────────────
// These return shell command strings to be executed outside raw mode.

/// Build the command to compose a new message.
pub fn compose_command(account: Option<&str>) -> String {
    let mut args = vec![];
    args.extend(["message".to_string(), "write".to_string()]);
    args.extend(account_args(account));
    editor_command(&args)
}

/// Build the command to reply to a message.
pub fn reply_command(account: Option<&str>, folder: &str, id: &str, all: bool) -> String {
    let mut args = vec![];
    args.extend(["message".to_string(), "reply".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string()]);
    if all {
        args.push("-A".to_string());
    }
    args.push(id.to_string());
    editor_command(&args)
}

/// Build the command to forward a message.
pub fn forward_command(account: Option<&str>, folder: &str, id: &str) -> String {
    let mut args = vec![];
    args.extend(["message".to_string(), "forward".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    editor_command(&args)
}

// ── Template operations (non-interactive compose) ───────────────────
// These generate MML templates without launching $EDITOR.

/// Generate a blank compose template (headers + signature) for the account.
pub fn template_write(account: Option<&str>) -> Result<String> {
    let mut args = global_args();
    args.extend(["template".to_string(), "write".to_string()]);
    args.extend(account_args(account));
    run(&args)
}

/// Generate a reply template for the given message.
pub fn template_reply(account: Option<&str>, folder: &str, id: &str, all: bool) -> Result<String> {
    let mut args = global_args();
    args.extend(["template".to_string(), "reply".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string()]);
    if all {
        args.push("-A".to_string());
    }
    args.push(id.to_string());
    run(&args)
}

/// Generate a forward template for the given message.
pub fn template_forward(account: Option<&str>, folder: &str, id: &str) -> Result<String> {
    let mut args = global_args();
    args.extend(["template".to_string(), "forward".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    run(&args)
}

/// Send a compiled MML template via stdin.
///
/// himalaya's `template send` checks `io::stdin().is_terminal()`: when
/// invoked as a subprocess (non-interactive), it reads the template from
/// stdin and ignores positional args. We pipe the template directly.
pub fn template_send(account: Option<&str>, template: &str) -> Result<String> {
    let mut args = global_args();
    args.extend(["template".to_string(), "send".to_string()]);
    args.extend(account_args(account));
    run_with_stdin(&args, template)
}

// ── Command builder (for testing) ───────────────────────────────────

/// Build command args for list_envelopes (exposed for testing).
pub fn build_envelope_args(
    account: Option<&str>,
    folder: &str,
    page: usize,
    page_size: usize,
    query: Option<&str>,
) -> Vec<String> {
    let mut args = global_args();
    args.extend(["envelope".to_string(), "list".to_string()]);
    args.extend(account_args(account));
    args.extend([
        "-f".to_string(),
        folder.to_string(),
        "-p".to_string(),
        page.to_string(),
        "-s".to_string(),
        page_size.to_string(),
    ]);
    if let Some(q) = query {
        for word in q.split_whitespace() {
            args.push(word.to_string());
        }
    }
    args
}

/// Build command args for read_message (exposed for testing).
pub fn build_read_args(account: Option<&str>, folder: &str, id: &str) -> Vec<String> {
    let mut args = global_args();
    args.extend(["message".to_string(), "read".to_string()]);
    args.extend(account_args(account));
    args.extend(["-f".to_string(), folder.to_string(), id.to_string()]);
    args
}
