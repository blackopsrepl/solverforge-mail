mod account_list;
mod envelope_list;
mod folder_list;
mod help;
mod message_view;
mod move_prompt;
mod search;
mod status_bar;

use ratatui::prelude::*;

use crate::app::App;
use crate::keys::View;

/// Main render dispatch — called once per frame.
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Top-level vertical layout: header (1), main body (fill), status bar (1)
    let outer = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    // ── Header ──────────────────────────────────────────────────
    status_bar::render_header(app, frame, outer[0]);

    // ── Main body ───────────────────────────────────────────────
    match app.view {
        View::MessageView => {
            // Full-width message view (no sidebar)
            message_view::render(app, frame, outer[1]);
        }
        _ => {
            // Sidebar + envelope list
            let main =
                Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)]).split(outer[1]);

            folder_list::render(app, frame, main[0]);
            envelope_list::render(app, frame, main[1]);
        }
    }

    // ── Status / search / move bar ──────────────────────────────
    match app.view {
        View::Search => search::render(app, frame, outer[2]),
        View::MovePrompt => move_prompt::render(app, frame, outer[2]),
        _ => status_bar::render(app, frame, outer[2]),
    }

    // ── Overlays ────────────────────────────────────────────────
    if app.view == View::AccountList {
        account_list::render(app, frame);
    }
    if app.view == View::Help {
        help::render(app, frame);
    }
}
