//! Identity list overlay — shows all identities for the current account.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use crate::app::App;
use crate::theme::theme;

pub fn render(app: &App, frame: &mut Frame) {
    let t = theme();
    let area = frame.area();

    // Center a popup: 50 cols wide, up to 60% of height
    let popup_w = area.width.min(56);
    let list_rows = (app.identities.len() as u16).max(1);
    // height: 2 borders + 1 title + list rows + 1 hint = list_rows + 4, capped
    let popup_h = (list_rows + 4).min(area.height * 60 / 100).max(6);
    let popup = Rect {
        x: area.x + area.width.saturating_sub(popup_w) / 2,
        y: area.y + area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let account_label = app.account_name.as_deref().unwrap_or("?");
    let title = format!(" Identities: {account_label} ");

    let block = Block::default()
        .title(Span::styled(title, t.popup_title()))
        .borders(Borders::ALL)
        .border_style(t.border_focused())
        .style(t.popup());

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Split inner: list (fill) + hint bar (1 row)
    let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(inner);

    // ── Identity list ─────────────────────────────────────────────────
    if app.identities.is_empty() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Span::styled(
                "  No identities. Press n to add one.",
                t.dimmed(),
            )),
            layout[0],
        );
    } else {
        let items: Vec<ListItem> = app
            .identities
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let default_marker = if id.is_default { " [default]" } else { "" };
                let label = format!("  {}{}", id.label(), default_marker);
                let style = if Some(i) == app.identity_index {
                    t.selected()
                } else {
                    t.normal()
                };
                ListItem::new(label).style(style)
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(app.identity_index);

        let list = List::new(items);
        frame.render_stateful_widget(list, layout[0], &mut list_state);
    }

    // ── Hint bar ──────────────────────────────────────────────────────
    frame.render_widget(
        ratatui::widgets::Paragraph::new(Span::styled(
            " n: new  e: edit  d: del  s: default  q: close ",
            t.dimmed(),
        ))
        .alignment(Alignment::Center),
        layout[1],
    );
}
