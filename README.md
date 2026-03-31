# SolverForge Mail

A spiffy ratatui-based TUI email client that wraps the himalaya CLI.

## Quick Start

```bash
# Run with auto-detected account
./solverforge-mail

# Run with specific account
./solverforge-mail test
./solverforge-mail --account icloud

# Set up accounts
./setup-accounts.sh
```

## Features

- **Non-blocking I/O** - Background workers for all himalaya operations
- **Relative timestamps** - "2h ago", "Yesterday", "Mon"
- **Threading support** - Press `t` to toggle threaded view
- **Auto-refresh** - New mail check every 60 seconds
- **Folder unread counts** - Shows (3) badge on folders
- **Mouse support** - Click to select, scroll wheel works
- **Multi-account** - Switch with Ctrl+a
- **Vim-style navigation** - j/k and g/G in list/message views, plus edtui keys in compose body
- **Smart error handling** - ANSI stripping, clean error messages
- **Address book** - Contacts with name, email, phone, org, notes, tags
- **Contact import** - vCard (.vcf) and Google CSV import
- **Auto-harvest contacts** - Captured from sent/received mail
- **Sender identities** - Multiple From addresses per account with default
- **Local SQLite database** - Contacts, identities, credentials stored in `~/.local/share/solverforge/mail.db`
- **Credential management** - Passwords and OAuth tokens in DB (no keyring dependency)

## Keybindings

### Global
- `Ctrl+c` / `Ctrl+q` - Quit
- `Ctrl+a` - Switch account
- `Ctrl+r` - Refresh
- `?` - Help

### Envelope List
- `j`/`k` - Navigate up/down
- `Enter` - Read message
- `c` - Compose new
- `d` - Delete
- `m` - Move to folder
- `!` - Toggle flag
- `t` - Toggle threaded view
- `/` - Search
- `Tab` - Focus folders
- `Ctrl+b` - Open contacts
- `I` - Open identities

### Message View
- `j`/`k` - Scroll
- `q`/`Esc` - Back to list
- `r` - Reply
- `R` - Reply all
- `f` - Forward
- `d` - Delete
- `a` - Download attachments

### Compose View
- `Tab` / `Shift+Tab` - Next/previous compose field
- `j`/`k` in **Nav mode** (headers/action bar) - Next/previous field
- `Enter` - Activate focused control (enter insert mode, cycle From, or activate action button)
- Typing in header text fields auto-enters Insert mode and inserts the character
- `Esc` - Exit insert mode / leave action bar focus
- `Ctrl+c` / `Ctrl+q` - Discard compose
- In **Body** focus: keys are forwarded to edtui (vim-style body editing stays local to body)
- Discard confirmation modal: `y` confirms, `n`/`Esc` cancels

### Mouse
- Scroll wheel - Navigate/scroll
- Left click - Select folder/envelope
- Right click - Go back (in message view)

## Input Architecture (Compose)

Compose input is resolved in two layers:

1. **Context builder (`App::compose_key_context`)** maps runtime compose state into a compact context:
   - Focus bucket: `From` / `Header` / `Body` / `ActionBar`
   - Edit mode: `Nav` / `Insert`
   - Popup flags: autocomplete visible, discard-confirm visible
2. **Contextual resolver (`resolve_compose_with_context`)** applies deterministic priority rules:
   - Global compose shortcuts (`Ctrl+c`, `Ctrl+q`)
   - Discard-confirm modal interception (`y`, `n`, `Esc`)
   - Autocomplete navigation/accept interception
   - Body passthrough to edtui (except `Tab` / `Shift+Tab` for field cycling)
   - Header/action-bar navigation and editing actions

### Why this design
- Keeps **vim-style body editing local to Body focus** (no interface-wide modal leakage).
- Keeps compose behavior explicit and testable with a single resolver function.
- Makes modal interactions predictable by using a clear precedence order.

### Best-practice target outcome
For this use case, the ideal architecture is:
- A **single authoritative input router per view** (Compose already follows this pattern).
- State modeled as explicit finite modes + overlays (focus + edit mode + popups).
- Pure key-resolution functions with unit tests for each mode interaction.
- Minimal side effects in key resolver; side effects happen in `App` action handlers.

### One-pass, low-regression delivery strategy
To improve safely in one pass:
1. Keep behavior changes isolated to the compose resolver (`resolve_compose_with_context`).
2. Encode precedence explicitly (shortcut > modal > popup > body passthrough > header/action).
3. Add regression tests for each precedence boundary.
4. Avoid moving side-effectful logic into resolver code.

## Account Setup

The app supports multiple account types:

### Test Account (Local Maildir)
Already configured with sample emails:
```bash
./solverforge-mail test
```

### Real Accounts
Run the setup wizard:
```bash
./setup-accounts.sh
```

Individual account setup:
- **iCloud**: Requires app-specific password from appleid.apple.com
- **Blinkenshell**: Simple password authentication
- **Gmail/Outlook**: OAuth2 browser flow

## Architecture

- **TEA pattern** - The Elm Architecture (Model, Update, View)
- **Async worker pool** - Background threads for all I/O
- **Channel-based IPC** - mpsc for result passing
- **Theme support** - Reads SolverForge colors.toml
- **Zero dependencies** on async runtime (no tokio)

## Stats

- 35 files, 12518 lines of Rust
- 81 tests, all passing
- 4.6MB release binary
- Zero warnings

## Troubleshooting

### No accounts working
The test account always works:
```bash
./solverforge-mail test
```

### Authentication errors
- **iCloud**: Need app-specific password, not Apple ID password
- **Gmail/Outlook**: OAuth tokens expire, re-run `himalaya account configure`
- **Blinkenshell**: Check keyring is unlocked (`kwalletd6` running)

### Keyring issues
```bash
# Check if keyring is accessible
secret-tool store --label="test" service test user test
# (enter any password)
secret-tool lookup service test user test
```

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Run with specific account
cargo run -- --account test
```

## Files

```
solverforge-mail/
├── solverforge-mail          # Smart launcher (auto-detects working account)
├── setup-accounts.sh         # Interactive account setup wizard
├── setup-blinkenshell.sh     # Blinkenshell password setup
├── setup-icloud.sh           # iCloud app-specific password setup
├── setup-oauth.sh            # Gmail/Outlook OAuth setup
├── src/
│   ├── main.rs              # Entry point, terminal setup
│   ├── app.rs               # TEA state machine
│   ├── worker.rs            # Background thread pool
│   ├── event.rs             # Terminal event handling
│   ├── keys.rs              # Keybinding definitions
│   ├── theme.rs             # Color theme loader
│   ├── himalaya/
│   │   ├── client.rs        # Himalaya CLI wrapper
│   │   ├── config.rs        # Binary discovery
│   │   └── types.rs         # JSON types
│   └── ui/
│       ├── envelope_list.rs # Email list with relative dates
│       ├── folder_list.rs   # Sidebar with unread counts
│       ├── message_view.rs  # Email reader
│       ├── account_list.rs  # Account switcher
│       └── ...              # Other UI components
└── tests/                   # 54 comprehensive tests
```
