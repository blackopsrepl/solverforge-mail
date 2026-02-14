use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::app::App;
use crate::keys::View;
use crate::theme::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let t = theme();
    let focused = app.view == View::EnvelopeList;

    let border_style = if focused {
        t.border_focused()
    } else {
        t.border()
    };

    let title = if let Some(ref q) = app.active_query {
        format!(" {} \u{2014} search: {q} ", app.current_folder) // —
    } else {
        format!(" {} \u{2014} p{} ", app.current_folder, app.page)
    };

    let block = Block::default()
        .title(title)
        .title_style(if focused {
            t.accent_style().add_modifier(Modifier::BOLD)
        } else {
            t.dimmed()
        })
        .borders(Borders::ALL)
        .border_style(border_style);

    if app.envelopes.is_empty() {
        let empty = Table::new(Vec::<Row>::new(), &[Constraint::Fill(1)]).block(block);
        frame.render_widget(empty, area);
        return;
    }

    let header_cells = ["", "From", "Subject", "Date"]
        .iter()
        .map(|h| Cell::from(*h).style(t.dimmed().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .envelopes
        .iter()
        .map(|env| {
            let base_style = if env.is_flagged() {
                t.flagged()
            } else if !env.is_seen() {
                t.unread()
            } else {
                t.normal()
            };

            let flag_cell = Cell::from(env.flag_icon()).style(base_style);
            let from_cell = Cell::from(truncate(&env.sender_display(), 24)).style(base_style);
            let subject_cell = Cell::from(env.subject.as_str()).style(base_style);
            let date_cell = Cell::from(short_date(&env.date)).style(t.dimmed());

            Row::new(vec![flag_cell, from_cell, subject_cell, date_cell])
        })
        .collect();

    // Column widths: flag(2), from(24), subject(fill), date(18)
    let widths = [
        Constraint::Length(2),
        Constraint::Length(24),
        Constraint::Fill(1),
        Constraint::Length(18),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(t.selected());

    frame.render_stateful_widget(table, area, &mut app.envelope_state);
}

/// Truncate a string to `max` characters, appending \u{2026} if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('\u{2026}'); // …
        out
    }
}

/// Shorten an ISO date to a friendlier form.
fn short_date(date: &str) -> String {
    // himalaya dates are like "2026-02-14 10:30:00+00:00" or similar
    // Take the first 16 chars at most.
    if date.len() >= 16 {
        date[..16].to_string()
    } else {
        date.to_string()
    }
}
