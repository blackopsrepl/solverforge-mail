//! Contact add/edit form UI.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::contact_edit::ContactField;
use crate::keys::EditMode;
use crate::theme::theme;
use crate::ui::action_bar::{render_action_bar, Button, ICON_DISCARD, ICON_SAVE};

/// Render the contact add/edit form as a centered popup overlay.
pub fn render(app: &App, frame: &mut Frame) {
    let t = theme();
    let area = frame.area();

    let Some(ref state) = app.contact_edit_state else {
        return;
    };

    // ── Backdrop dim ─────────────────────────────────────────────────
    frame.render_widget(Clear, area);

    // ── Center a popup ────────────────────────────────────────────────
    let popup_w = area.width.min(60);
    let popup_h = 20u16;
    let popup = Rect {
        x: area.x + area.width.saturating_sub(popup_w) / 2,
        y: area.y + area.height.saturating_sub(popup_h) / 2,
        width: popup_w,
        height: popup_h,
    };

    let title = if state.contact_id.is_some() {
        " Edit Contact "
    } else {
        " New Contact "
    };

    let block = Block::default()
        .title(Span::styled(title, t.popup_title()))
        .borders(Borders::ALL)
        .border_style(t.border_focused())
        .style(t.popup());
    frame.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    // ── Fields ────────────────────────────────────────────────────────
    let fields = [
        (ContactField::Name, "Name   ", state.name.as_str()),
        (ContactField::Email, "Email  ", state.email.as_str()),
        (ContactField::Phone, "Phone  ", state.phone.as_str()),
        (ContactField::Org, "Org    ", state.org.as_str()),
        (ContactField::Notes, "Notes  ", state.notes.as_str()),
        (ContactField::Tags, "Tags   ", state.tags.as_str()),
    ];

    // Reserve the last 2 rows for error + action bar
    let fields_height = inner.height.saturating_sub(2);
    let field_height = 2u16; // label + input row

    for (row_idx, (field, label, value)) in fields.iter().enumerate() {
        let y = row_idx as u16 * field_height;
        if y + field_height > fields_height {
            break;
        }

        let focused = state.focused == *field;
        let label_area = Rect {
            x: inner.x,
            y: inner.y + y,
            width: inner.width,
            height: 1,
        };
        let input_area = Rect {
            x: inner.x,
            y: inner.y + y + 1,
            width: inner.width,
            height: 1,
        };

        // Label
        frame.render_widget(
            Paragraph::new(Span::styled(*label, t.header_label())),
            label_area,
        );

        // Input value with cursor if focused
        let display = if focused {
            format!("{}_", value)
        } else {
            value.to_string()
        };
        let style = if focused { t.selected() } else { t.normal() };
        frame.render_widget(
            Paragraph::new(Span::styled(display, style)).block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(if focused {
                        t.border_focused()
                    } else {
                        t.border()
                    }),
            ),
            input_area,
        );
    }

    // ── Error bar ─────────────────────────────────────────────────────
    let action_bar_y = inner.y + inner.height.saturating_sub(1);
    let error_y = action_bar_y.saturating_sub(1);

    if let Some(ref err) = state.error {
        frame.render_widget(
            Paragraph::new(Span::styled(err.as_str(), t.error())).alignment(Alignment::Center),
            Rect {
                x: inner.x,
                y: error_y,
                width: inner.width,
                height: 1,
            },
        );
    }

    // ── Action bar (last row) ────────────────────────────────────────
    let action_area = Rect {
        x: inner.x,
        y: action_bar_y,
        width: inner.width,
        height: 1,
    };

    let focused_btn_idx = match state.focused {
        ContactField::Save => Some(0usize),
        ContactField::Cancel => Some(1),
        _ => None,
    };

    let buttons = [
        Button {
            label: &format!("{} Save", ICON_SAVE),
            focused: state.focused == ContactField::Save,
            disabled: false,
        },
        Button {
            label: &format!("{} Cancel", ICON_DISCARD),
            focused: state.focused == ContactField::Cancel,
            disabled: false,
        },
    ];

    render_action_bar(frame, action_area, EditMode::Nav, &buttons, focused_btn_idx);
}
