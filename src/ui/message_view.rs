use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let t = theme();

    let block = Block::default()
        .title(" Message ")
        .title_style(t.accent_style().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(t.border_focused());

    let body = &app.message_body;

    // Build styled lines: headers get special treatment, body is normal.
    let mut lines: Vec<Line> = Vec::new();
    let mut in_headers = true;

    for raw_line in body.lines() {
        if in_headers {
            if raw_line.is_empty() {
                // Separator between headers and body
                in_headers = false;
                lines.push(Line::from(Span::styled(
                    "\u{2500}".repeat(area.width.saturating_sub(4) as usize),
                    t.dimmed(),
                )));
                continue;
            }
            // Try to split header label from value
            if let Some((label, value)) = raw_line.split_once(':') {
                lines.push(Line::from(vec![
                    Span::styled(format!("{label}: "), t.header_label()),
                    Span::styled(value.trim_start().to_string(), t.header_value()),
                ]));
            } else {
                lines.push(Line::from(Span::styled(raw_line.to_string(), t.normal())));
            }
        } else {
            // Message body — detect quoted lines
            let style = if raw_line.starts_with('>') {
                t.dimmed()
            } else {
                t.normal()
            };
            lines.push(Line::from(Span::styled(raw_line.to_string(), style)));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.message_scroll, 0));

    frame.render_widget(paragraph, area);
}
