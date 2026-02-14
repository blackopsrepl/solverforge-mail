#![allow(dead_code)]

mod app;
mod event;
mod himalaya;
mod keys;
mod theme;
mod ui;

use std::io::{self, stdout};
use std::panic;
use std::time::Duration;

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use app::App;
use event::EventHandler;

fn main() -> Result<()> {
    // ── Parse minimal CLI args ──────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    let account = parse_account_flag(&args);

    // ── Install panic hook that restores the terminal ───────────────
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    // ── Terminal setup ──────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── Run ─────────────────────────────────────────────────────────
    let events = EventHandler::new(Duration::from_millis(250));
    let mut app = App::new(account);
    app.init();
    let result = run(&mut terminal, &mut app, &events);

    // ── Teardown ────────────────────────────────────────────────────
    restore_terminal()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    while app.running {
        // Render
        terminal.draw(|frame| ui::render(app, frame))?;

        // Handle next event
        match events.next()? {
            event::Event::Key(key) => app.handle_key(key),
            event::Event::Tick => app.tick(),
            event::Event::Resize(_, _) => {} // ratatui handles resize
        }

        // If app requested a shell-out (compose/reply/forward), do it
        // between frames while we still own the terminal.
        if let Some(cmd) = app.pending_shell.take() {
            restore_terminal()?;
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .status();
            // Re-enter raw/alternate mode
            enable_raw_mode()?;
            execute!(io::stdout(), EnterAlternateScreen)?;
            terminal.clear()?;
            match status {
                Ok(s) if s.success() => app.set_status("Message sent."),
                Ok(s) => app.set_status(&format!("Editor exited with {s}")),
                Err(e) => app.set_status(&format!("Failed to launch editor: {e}")),
            }
            // Refresh envelope list after compose actions
            app.refresh_envelopes();
        }
    }
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn parse_account_flag(args: &[String]) -> Option<String> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--account" || arg == "-a" {
            return iter.next().cloned();
        }
        if let Some(val) = arg.strip_prefix("--account=") {
            return Some(val.to_string());
        }
    }
    None
}
