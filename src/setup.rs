use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};

use crate::himalaya::client;
use crate::mail::types::Account;
use crate::mail::{default_mail_service, MailService};

pub fn run_wizard() -> Result<Option<String>> {
    println!("╔════════════════════════════════════════════╗");
    println!("║     SolverForge Mail - Account Setup      ║");
    println!("╚════════════════════════════════════════════╝");
    println!();

    loop {
        let inventory = load_inventory()?;
        print_inventory(&inventory);

        if should_show_bootstrap_only(&inventory.accounts) {
            println!();
            println!("1) Bootstrap an OAuth account");
            println!("2) Exit");
            println!();

            match prompt("Choice [1-2]: ")? {
                choice if choice == "1" => configure_oauth(None)?,
                choice if choice == "2" || choice.is_empty() => return Ok(None),
                _ => println!("Invalid choice."),
            }

            println!();
            continue;
        }

        println!();
        println!("Select an action:");
        println!("1) Store secrets for a password-based IMAP/SMTP account");
        println!("2) Configure iCloud credentials");
        println!("3) Run the OAuth browser flow");
        println!("4) Launch SolverForge Mail with the first working account");
        println!("5) Exit");
        println!();

        match prompt("Choice [1-5]: ")? {
            choice if choice == "1" => {
                if let Err(error) = configure_password_account(&inventory.accounts) {
                    println!("{error}");
                }
            }
            choice if choice == "2" => {
                if let Err(error) = configure_icloud_account(&inventory.accounts) {
                    println!("{error}");
                }
            }
            choice if choice == "3" => configure_oauth(Some(&inventory.accounts))?,
            choice if choice == "4" => {
                return Ok(Some(first_working_account(&inventory.accounts)?))
            }
            choice if choice == "5" || choice.is_empty() => return Ok(None),
            _ => println!("Invalid choice."),
        }

        println!();
    }
}

pub fn print_account_status() -> Result<()> {
    let inventory = load_inventory()?;
    print_inventory(&inventory);

    if inventory.accounts.is_empty() {
        if let Some(message) = inventory.banner.as_ref() {
            bail!("{message}");
        }
        bail!("No configured accounts found.");
    }

    Ok(())
}

struct Inventory {
    accounts: Vec<Account>,
    banner: Option<String>,
}

fn load_inventory() -> Result<Inventory> {
    let accounts = mail_service()
        .list_accounts()
        .map_err(|error| anyhow!(error.to_string()))?;
    let banner = if should_show_bootstrap_only(&accounts) {
        Some("No configured remote accounts found. OAuth bootstrap can create the first remote account from here.".to_string())
    } else {
        None
    };
    Ok(Inventory { accounts, banner })
}

fn print_inventory(inventory: &Inventory) {
    println!("Current account status:");
    println!("----------------------");

    if let Some(message) = inventory.banner.as_ref() {
        println!("{message}");
        return;
    }

    for account in &inventory.accounts {
        println!(
            "{} {}",
            pad_account(&account.name),
            describe_account_status(account)
        );
    }
}

fn pad_account(name: &str) -> String {
    format!("{name:20}:")
}

fn describe_account_status(account: &Account) -> String {
    match mail_service().probe_account(&account.name) {
        Ok(()) => "✓ Working".to_string(),
        Err(error) => format!("✗ {error}"),
    }
}

fn configure_password_account(accounts: &[Account]) -> Result<()> {
    let account = choose_existing_account(accounts, "Password account to configure")?;
    let username = prompt_nonempty(&format!("Username/login for {}: ", account.name))?;
    let password = prompt_password(&format!("Password for {}: ", account.name))?;

    store_secret(
        &format!("{} IMAP password", account.name),
        &format!("{}-imap", account.name),
        &username,
        &password,
    )?;
    store_secret(
        &format!("{} SMTP password", account.name),
        &format!("{}-smtp", account.name),
        &username,
        &password,
    )?;

    println!("Stored keyring secrets for {}.", account.name);
    print_probe_result(&account.name, Some(&account.backend));
    Ok(())
}

fn configure_icloud_account(accounts: &[Account]) -> Result<()> {
    let account = choose_existing_account(accounts, "iCloud account to configure")?;
    let email = prompt_nonempty("iCloud email address: ")?;
    let password = prompt_password("iCloud app-specific password: ")?;

    if authinfo_gpg_path().is_file() {
        let recipient = prompt_nonempty("GPG recipient for ~/.authinfo.gpg: ")?;
        rewrite_authinfo_gpg(&email, &password, &recipient)?;
        println!("Updated ~/.authinfo.gpg.");
    } else {
        store_secret(
            &format!("{} IMAP password", account.name),
            &format!("{}-imap", account.name),
            &email,
            &password,
        )?;
        store_secret(
            &format!("{} SMTP password", account.name),
            &format!("{}-smtp", account.name),
            &email,
            &password,
        )?;
        println!("Stored keyring secrets for {}.", account.name);
    }

    print_probe_result(&account.name, Some(&account.backend));
    Ok(())
}

fn configure_oauth(accounts: Option<&[Account]>) -> Result<()> {
    let account_name = choose_oauth_account(accounts)?;
    println!("Starting OAuth setup for {}...", account_name);
    client::configure_account(&account_name)?;
    print_probe_result(
        &account_name,
        accounts.and_then(|items| find_backend(items, &account_name)),
    );
    Ok(())
}

fn first_working_account(accounts: &[Account]) -> Result<String> {
    let mut candidates = accounts.to_vec();
    crate::mail::types::sort_accounts(&mut candidates);

    for account in &candidates {
        if mail_service().probe_account(&account.name).is_ok() {
            println!("Launching SolverForge Mail with account {}.", account.name);
            return Ok(account.name.clone());
        }
    }

    bail!("No working account found. Fix the reported backend/auth issues first.")
}

fn choose_existing_account<'a>(accounts: &'a [Account], prompt_text: &str) -> Result<&'a Account> {
    let selectable = selectable_remote_accounts(accounts);
    if selectable.is_empty() {
        bail!("No remote accounts are available for this setup flow yet.");
    }

    for (index, account) in selectable.iter().enumerate() {
        let default_marker = if account.default { " (default)" } else { "" };
        println!(
            "  {}) {} [{}]{}",
            index + 1,
            account.name,
            account.backend,
            default_marker
        );
    }
    println!();

    let raw = prompt(&format!("{prompt_text} [1-{}]: ", selectable.len()))?;
    let choice: usize = raw.parse().context("invalid account selection")?;
    selectable
        .get(choice.saturating_sub(1))
        .copied()
        .ok_or_else(|| anyhow!("account selection out of range"))
}

fn choose_oauth_account(accounts: Option<&[Account]>) -> Result<String> {
    if let Some(accounts) = accounts {
        let selectable = selectable_remote_accounts(accounts);
        for (index, account) in selectable.iter().enumerate() {
            let default_marker = if account.default { " (default)" } else { "" };
            println!(
                "  {}) {} [{}]{}",
                index + 1,
                account.name,
                account.backend,
                default_marker
            );
        }
        println!("  0) Enter a new account name");
        println!();

        let raw = prompt(&format!("OAuth account [0-{}]: ", selectable.len()))?;
        if raw == "0" {
            return prompt_nonempty("New account name: ");
        }

        let choice: usize = raw.parse().context("invalid account selection")?;
        let account = selectable
            .get(choice.saturating_sub(1))
            .copied()
            .ok_or_else(|| anyhow!("account selection out of range"))?;
        return Ok(account.name.clone());
    }

    prompt_nonempty("Account name to configure: ")
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read from stdin")?;
    Ok(line.trim().to_string())
}

fn prompt_nonempty(label: &str) -> Result<String> {
    let value = prompt(label)?;
    if value.is_empty() {
        bail!("input is required");
    }
    Ok(value)
}

fn prompt_password(label: &str) -> Result<String> {
    let value = rpassword::prompt_password(label).context("failed to read password")?;
    if value.is_empty() {
        bail!("password is required");
    }
    Ok(value)
}

fn print_probe_result(account: &str, _backend: Option<&str>) {
    match mail_service().probe_account(account) {
        Ok(()) => println!("✓ {} is working.", account),
        Err(error) => println!("✗ {error}"),
    }
}

fn find_backend<'a>(accounts: &'a [Account], account_name: &str) -> Option<&'a str> {
    accounts
        .iter()
        .find(|account| account.name == account_name)
        .map(|account| account.backend.as_str())
}

fn selectable_remote_accounts(accounts: &[Account]) -> Vec<&Account> {
    accounts
        .iter()
        .filter(|account| !account.backend.eq_ignore_ascii_case("maildir"))
        .collect()
}

fn should_show_bootstrap_only(accounts: &[Account]) -> bool {
    selectable_remote_accounts(accounts).is_empty()
}

fn mail_service() -> Arc<dyn MailService> {
    default_mail_service()
}

fn store_secret(label: &str, service: &str, username: &str, password: &str) -> Result<()> {
    let mut child = Command::new("secret-tool")
        .args([
            "store",
            "--label",
            label,
            "service",
            service,
            "username",
            username,
            "application",
            "himalaya",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to execute secret-tool for service {}", service))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(password.as_bytes())
            .with_context(|| format!("failed to write secret for service {}", service))?;
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for secret-tool")?;

    if !output.status.success() {
        bail!(
            "secret-tool failed for service {}: {}",
            service,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

fn authinfo_gpg_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".authinfo.gpg")
}

fn rewrite_authinfo_gpg(email: &str, password: &str, recipient: &str) -> Result<()> {
    let authinfo = authinfo_gpg_path();
    let decrypted = Command::new("gpg")
        .args(["-q", "--for-your-eyes-only", "-d"])
        .arg(&authinfo)
        .output()
        .with_context(|| format!("failed to read {}", authinfo.display()))?;

    let mut lines = if decrypted.status.success() {
        String::from_utf8_lossy(&decrypted.stdout)
            .lines()
            .filter(|line| !line.contains("imap.mail.me.com") && !line.contains("smtp.mail.me.com"))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    lines.push(format!(
        "machine imap.mail.me.com login {} password {}",
        email, password
    ));
    lines.push(format!(
        "machine smtp.mail.me.com login {} password {}",
        email, password
    ));

    let tmp =
        std::env::temp_dir().join(format!("solverforge-mail-authinfo-{}", std::process::id()));
    fs::write(&tmp, lines.join("\n") + "\n")
        .with_context(|| format!("failed to write {}", tmp.display()))?;

    let status = Command::new("gpg")
        .args(["--batch", "--yes", "-e", "-r", recipient])
        .arg(&tmp)
        .status()
        .context("failed to execute gpg")?;
    if !status.success() {
        let _ = fs::remove_file(&tmp);
        bail!("gpg encryption failed");
    }

    let encrypted = tmp.with_extension("gpg");
    fs::rename(&encrypted, &authinfo).with_context(|| {
        format!(
            "failed to move encrypted authinfo from {} to {}",
            encrypted.display(),
            authinfo.display()
        )
    })?;
    let _ = fs::remove_file(&tmp);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{selectable_remote_accounts, should_show_bootstrap_only};
    use crate::mail::types::Account;

    #[test]
    fn only_test_account_uses_bootstrap_menu() {
        let accounts = vec![Account {
            name: "test".to_string(),
            backend: "maildir".to_string(),
            default: false,
        }];

        assert!(should_show_bootstrap_only(&accounts));
        assert!(selectable_remote_accounts(&accounts).is_empty());
    }

    #[test]
    fn remote_accounts_enable_full_setup_menu() {
        let accounts = vec![
            Account {
                name: "test".to_string(),
                backend: "maildir".to_string(),
                default: false,
            },
            Account {
                name: "work".to_string(),
                backend: "imap".to_string(),
                default: false,
            },
        ];

        assert!(!should_show_bootstrap_only(&accounts));
        assert_eq!(selectable_remote_accounts(&accounts).len(), 1);
    }
}
