//! Contact browser UI — address book view.
//!
//! Layout: searchable list on the left, detail pane on the right.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme::theme;

/// Render the contacts browser.
pub fn render(app: &App, frame: &mut Frame) {
    use crate::keys::View;
    let t = theme();
    let area = frame.area();

    // Top-level: header + body
    let outer = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(area);

    // ── Header ────────────────────────────────────────────────────────
    let title = format!(" \u{2605} address book  [{} contacts] ", app.contacts.len());
    let hints = if app.view == View::ContactSearch {
        " Enter/Esc: done · type to search "
    } else {
        " Ctrl+b: close · n: new · d: delete · e: edit · /: search "
    };
    let fill = " ".repeat(
        area.width
            .saturating_sub(title.chars().count() as u16 + hints.chars().count() as u16)
            as usize,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(title, t.header()),
            Span::styled(fill, t.header()),
            Span::styled(hints, t.header()),
        ])),
        outer[0],
    );

    // ── Body: list (40%) + detail (60%) ───────────────────────────────
    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    render_list(app, frame, body[0]);
    render_detail(app, frame, body[1]);
}

fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    use crate::keys::View;
    let t = theme();

    let title = if app.contact_search.is_empty() {
        " Contacts ".to_string()
    } else {
        let cursor = if app.view == View::ContactSearch {
            "█"
        } else {
            ""
        };
        format!(" Search: {}{} ", app.contact_search, cursor)
    };

    let block = Block::default()
        .title(Span::styled(title, t.popup_title()))
        .borders(Borders::ALL)
        .border_style(t.border_focused())
        .style(t.popup());

    let items: Vec<ListItem> = app
        .contacts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if Some(i) == app.contact_index {
                t.selected()
            } else {
                t.normal()
            };
            let label = match &c.name {
                Some(n) if !n.is_empty() => format!("{}\n{}", n, c.email),
                _ => c.email.clone(),
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_detail(app: &App, frame: &mut Frame, area: Rect) {
    let t = theme();

    let contact = app.contact_index.and_then(|i| app.contacts.get(i));

    let block = Block::default()
        .title(Span::styled(" Detail ", t.popup_title()))
        .borders(Borders::ALL)
        .border_style(t.border())
        .style(t.popup());

    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let Some(c) = contact else {
        frame.render_widget(
            Paragraph::new("No contact selected")
                .style(t.dimmed())
                .alignment(Alignment::Center),
            inner,
        );
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    if let Some(name) = &c.name {
        lines.push(Line::from(vec![
            Span::styled("Name:    ", t.header_label()),
            Span::styled(name, t.header_value()),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("Email:   ", t.header_label()),
        Span::styled(&c.email, t.accent_style()),
    ]));
    if let Some(phone) = &c.phone {
        lines.push(Line::from(vec![
            Span::styled("Phone:   ", t.header_label()),
            Span::styled(phone, t.header_value()),
        ]));
    }
    if let Some(org) = &c.org {
        lines.push(Line::from(vec![
            Span::styled("Org:     ", t.header_label()),
            Span::styled(org, t.header_value()),
        ]));
    }
    if !c.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Tags:    ", t.header_label()),
            Span::styled(c.tags.join(", "), t.dimmed()),
        ]));
    }
    if c.harvested {
        lines.push(Line::from(Span::styled("(auto-harvested)", t.dimmed())));
    }
    if let Some(notes) = &c.notes {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Notes:", t.header_label())));
        for note_line in notes.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {note_line}"),
                t.normal(),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
