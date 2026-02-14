use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::theme::theme;

/// Render the move-to-folder input bar (replaces status bar when active).
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let t = theme();

    let cursor_char = if app.tick_count % 4 < 2 {
        "\u{2588}"
    } else {
        " "
    };

    let spans = vec![
        Span::styled(" Move to: ", t.status_key()),
        Span::styled(
            format!("{}{cursor_char}", app.move_target),
            t.search_input(),
        ),
        Span::styled(
            "  (type folder name, Enter to confirm, Esc to cancel)",
            t.dimmed(),
        ),
    ];

    let paragraph = Paragraph::new(Line::from(spans)).style(t.status_bar());
    frame.render_widget(paragraph, area);
}
