use solverforge_mail::himalaya::client;
use solverforge_mail::himalaya::types::*;

fn main() {
    println!("Testing himalaya commands directly...");

    // Test account list
    println!("\n1. Testing account list:");
    match client::list_accounts() {
        Ok(accounts) => {
            for acc in &accounts {
                println!("  - {} (default: {})", acc.name, acc.default);
            }
        }
        Err(e) => println!("  ERROR: {}", e),
    }

    // Test folder list for kgmail
    println!("\n2. Testing folder list for kgmail:");
    match client::list_folders(Some("kgmail")) {
        Ok(folders) => {
            for folder in &folders {
                println!("  - {}", folder.name);
            }
        }
        Err(e) => println!("  ERROR: {}", e),
    }

    // Test envelope list for kgmail INBOX
    println!("\n3. Testing envelope list for kgmail INBOX:");
    match client::list_envelopes(Some("kgmail"), "INBOX", 1, 10, None) {
        Ok(envelopes) => {
            println!("  Found {} envelopes:", envelopes.len());
            for env in envelopes.iter().take(5) {
                println!(
                    "    - [{}] {} - {}",
                    env.flags.join(","),
                    env.sender_display(),
                    env.subject
                );
            }
        }
        Err(e) => println!("  ERROR: {}", e),
    }
}
