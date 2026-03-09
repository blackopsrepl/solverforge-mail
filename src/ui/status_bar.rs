use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::keys;
use crate::theme::theme;

// Braille spinner frames.
const SPINNER: &[&str] = &[
    "\u{2801}", "\u{2809}", "\u{2819}", "\u{281b}", "\u{281e}", "\u{2836}", "\u{2834}", "\u{2824}",
];

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let t = theme();

    // Build key-hint spans
    let hints = keys::hints(app.view);
    let mut spans: Vec<Span> = Vec::new();

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", t.status_bar()));
        }
        spans.push(Span::styled(format!(" {key} "), t.status_key()));
        spans.push(Span::styled(desc.to_string(), t.status_desc()));
    }

    // Add status message or spinner on the right
    let right_content = if app.loading {
        let idx = (app.tick_count as usize) % SPINNER.len();
        format!(" {} ", SPINNER[idx])
    } else if !app.status_message.is_empty() {
        format!("  {}", app.status_message)
    } else {
        String::new()
    };

    let right_style = if app.status_is_error {
        t.error()
    } else if app.loading {
        t.spinner()
    } else {
        t.accent_style()
    };

    spans.push(Span::styled(right_content, right_style));

    let paragraph = Paragraph::new(Line::from(spans)).style(t.status_bar());
    frame.render_widget(paragraph, area);
}

/// Render the header bar at the top.
pub fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let t = theme();

    let account_label = app.account_name.as_deref().unwrap_or("(no account)");

    // New mail indicator
    let mail_badge = if app.new_mail_count > 0 {
        format!(" [{}]", app.new_mail_count)
    } else {
        String::new()
    };

    let thread_indicator = if app.threaded { " \u{2637}" } else { "" };

    let title = format!(
        "  \u{f0e0}  SolverForge Mail{mail_badge}          {account_label}  \u{2502}  {}{thread_indicator}  ",
        app.current_folder
    );

    let header = Paragraph::new(title).style(t.header());
    frame.render_widget(header, area);
}
