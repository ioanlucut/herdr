use super::*;

const TAB_SCROLL_BUTTON_WIDTH: u16 = 3;
// Text cells inside a card tab at most, so one long TLDR does not push every other tab away
const CARD_TEXT: u16 = 60;
const MIN_TAB_STRIP_WIDTH: u16 =
    MIN_TAB_WIDTH + NEW_TAB_WIDTH + TAB_SCROLL_BUTTON_WIDTH.saturating_mul(2);

pub(crate) fn render_tab_bar(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    tab_scroll: &mut usize,
    reveal_focused_tab: &mut bool,
    tab_drag_insert_index: Option<usize>,
    hits: &mut ShellHitMap,
) {
    let palette = &config.palette;
    buffer.set_style(area, Style::default().bg(palette.panel_bg));
    let tabs = snapshot
        .tabs
        .iter()
        .filter(|tab| Some(tab.workspace_id.as_str()) == snapshot.focused_workspace_id.as_deref())
        .collect::<Vec<_>>();
    let card = config.tab_style == crate::config::TabStyleConfig::Card && area.height >= 4;
    // Controls and the status area sit on the middle row, so they line up with a card's content
    let row = area.y + area.height / 2;
    let details = tabs
        .iter()
        .map(|tab| card.then(|| tab_detail(tab, snapshot, config)))
        .collect::<Vec<_>>();
    let desired_widths = tabs
        .iter()
        .zip(&details)
        .map(|(tab, detail)| match detail {
            // Two border cells and the padding around the longer of the label (with a space on
            // each side of it on the top border) and the detail, the text capped at CARD_TEXT
            Some(detail) => (display_width(&tab_label(tab)) + 2)
                .max(display_width(detail))
                .min(CARD_TEXT)
                .saturating_add(config.tab_padding.saturating_mul(2) + 2)
                .max(MIN_TAB_WIDTH),
            None => display_width(&tab_label(tab))
                .saturating_add(config.tab_padding.saturating_mul(2))
                .max(MIN_TAB_WIDTH),
        })
        .collect::<Vec<_>>();
    let content = tab_bar_content_area(snapshot, area);
    let mouse_chrome = config.mouse_capture;
    let new_tab_width = if mouse_chrome { NEW_TAB_WIDTH } else { 0 };
    let desired_total = desired_widths
        .iter()
        .copied()
        .fold(0_u16, u16::saturating_add)
        .saturating_add(tabs.len().saturating_sub(1).min(u16::MAX as usize) as u16)
        .saturating_add(new_tab_width);
    let overflow =
        desired_total > content.width && (!mouse_chrome || content.width >= MIN_TAB_STRIP_WIDTH);
    let available = if overflow && mouse_chrome {
        content
            .width
            .saturating_sub(NEW_TAB_WIDTH)
            .saturating_sub(TAB_SCROLL_BUTTON_WIDTH.saturating_mul(2))
    } else {
        content.width.saturating_sub(new_tab_width)
    };
    let max_scroll = max_tab_scroll(&desired_widths, available);
    if !overflow {
        *tab_scroll = 0;
    } else if *reveal_focused_tab {
        if let Some(focused) = tabs.iter().position(|tab| tab.focused) {
            *tab_scroll = centered_tab_scroll(focused, &desired_widths, available).min(max_scroll);
        }
    } else {
        *tab_scroll = (*tab_scroll).min(max_scroll);
    }
    *reveal_focused_tab = false;

    let mut x = content.x;
    let tab_right = if overflow && mouse_chrome {
        hits.tab_scroll_left = Rect::new(
            content.x,
            row,
            TAB_SCROLL_BUTTON_WIDTH.min(content.width),
            1,
        );
        put_text(
            buffer,
            hits.tab_scroll_left.x,
            row,
            hits.tab_scroll_left.width,
            " < ",
            Style::default()
                .fg(if *tab_scroll > 0 {
                    palette.overlay1
                } else {
                    palette.overlay0
                })
                .bg(palette.surface0),
        );
        x = hits.tab_scroll_left.right();
        content
            .right()
            .saturating_sub(NEW_TAB_WIDTH + TAB_SCROLL_BUTTON_WIDTH)
    } else {
        content.right().saturating_sub(new_tab_width)
    };

    let mut first_visible = None;
    let mut last_visible = None;
    for (index, tab) in tabs.iter().enumerate().skip(*tab_scroll) {
        let name = tab_label(tab);
        let desired = desired_widths[index];
        let remaining = tab_right.saturating_sub(x);
        let width = desired.min(remaining);
        if width == 0 {
            break;
        }
        // The tab spans every tab bar row, so a click on its detail row selects it too
        let rect = Rect::new(x, area.y, width, area.height);
        if let Some(detail) = &details[index] {
            draw_card(buffer, rect, tab, &name, detail, config);
        } else {
            let style = if tab.focused {
                let base = Style::default()
                    .fg(panel_contrast_fg(palette))
                    .bg(palette.accent);
                if tab.custom_label {
                    base.add_modifier(Modifier::BOLD)
                } else {
                    base
                }
            } else if tab.custom_label {
                Style::default().fg(palette.overlay1).bg(palette.surface0)
            } else {
                Style::default().fg(palette.overlay0).bg(palette.surface0)
            };
            let padding = width.saturating_sub(display_width(&name));
            let left = padding / 2;
            let text = format!(
                "{empty:left$}{name}{empty:right_padding$}",
                empty = "",
                left = left as usize,
                right_padding = padding.saturating_sub(left) as usize,
            );
            put_text(buffer, rect.x, rect.y, rect.width, &text, style);
        }
        hits.tabs.push((rect, tab.tab_id.clone()));
        first_visible.get_or_insert(index);
        last_visible = Some(index);
        x = x.saturating_add(width + 1);
        if width < desired {
            break;
        }
    }

    if overflow && mouse_chrome {
        hits.tab_scroll_right = Rect::new(tab_right, row, TAB_SCROLL_BUTTON_WIDTH, 1);
        let can_scroll_right = *tab_scroll < max_scroll;
        put_text(
            buffer,
            hits.tab_scroll_right.x,
            row,
            hits.tab_scroll_right.width,
            " > ",
            Style::default()
                .fg(if can_scroll_right {
                    palette.overlay1
                } else {
                    palette.overlay0
                })
                .bg(palette.surface0),
        );
        hits.new_tab = Rect::new(
            hits.tab_scroll_right.right(),
            row,
            content
                .right()
                .saturating_sub(hits.tab_scroll_right.right())
                .min(NEW_TAB_WIDTH),
            1,
        );
    } else if mouse_chrome {
        hits.new_tab = Rect::new(
            x.min(content.right()),
            row,
            content.right().saturating_sub(x).min(NEW_TAB_WIDTH),
            1,
        );
    }
    if mouse_chrome {
        put_text(
            buffer,
            hits.new_tab.x,
            row,
            hits.new_tab.width,
            " + ",
            Style::default().fg(palette.overlay1).bg(palette.panel_bg),
        );
    }

    if first_visible.is_some_and(|index| index > 0) {
        let ellipsis_x = if hits.tab_scroll_left.width > 0 {
            hits.tab_scroll_left.right()
        } else {
            content.x
        };
        put_text(
            buffer,
            ellipsis_x,
            row,
            u16::from(ellipsis_x < content.right()),
            "…",
            Style::default().fg(palette.overlay0),
        );
    }
    if last_visible.is_some_and(|index| index + 1 < tabs.len()) {
        let ellipsis_x = if hits.tab_scroll_right.width > 0 {
            hits.tab_scroll_right.x.saturating_sub(1)
        } else {
            content.right().saturating_sub(1)
        };
        put_text(
            buffer,
            ellipsis_x,
            row,
            u16::from(ellipsis_x >= content.x && ellipsis_x < content.right()),
            "…",
            Style::default().fg(palette.overlay0),
        );
    }

    if let Some(insert_index) = tab_drag_insert_index {
        if let Some(indicator_x) = tab_drop_indicator_x(hits, &tabs, insert_index) {
            put_text(
                buffer,
                indicator_x.min(content.right().saturating_sub(1)),
                row,
                1,
                "│",
                Style::default().fg(palette.accent),
            );
        }
    }
    render_tab_bar_status(buffer, area, snapshot, palette);
}

pub(crate) fn tab_bar_status_width(snapshot: &ClientShellSnapshot) -> u16 {
    let content = snapshot.tab_bar_right.iter().fold(0u16, |width, segment| {
        width.saturating_add(display_width(&segment.text))
    });
    let separators = snapshot.tab_bar_right.len().saturating_sub(1);
    content.saturating_add(
        display_width(&snapshot.tab_bar_right_separator)
            .saturating_mul(separators.min(u16::MAX as usize) as u16),
    )
}

fn tab_bar_status_area(snapshot: &ClientShellSnapshot, area: Rect) -> Option<Rect> {
    let width = tab_bar_status_width(snapshot);
    if width == 0 {
        return None;
    }
    let reserved = width.saturating_add(1);
    (area.width.saturating_sub(reserved) >= MIN_TAB_STRIP_WIDTH).then(|| {
        Rect::new(
            area.right().saturating_sub(width),
            area.y + area.height / 2,
            width,
            1,
        )
    })
}

fn tab_bar_content_area(snapshot: &ClientShellSnapshot, area: Rect) -> Rect {
    let reserved = tab_bar_status_area(snapshot, area)
        .map(|status| status.width.saturating_add(1))
        .unwrap_or(0);
    Rect {
        width: area.width.saturating_sub(reserved),
        ..area
    }
}

fn render_tab_bar_status(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    palette: &Palette,
) {
    let Some(status) = tab_bar_status_area(snapshot, area) else {
        return;
    };
    let separator_width = display_width(&snapshot.tab_bar_right_separator);
    let mut x = status.x;
    for (index, segment) in snapshot.tab_bar_right.iter().enumerate() {
        if index > 0 && separator_width > 0 {
            put_text(
                buffer,
                x,
                area.y,
                separator_width,
                &snapshot.tab_bar_right_separator,
                Style::default().fg(palette.overlay0).bg(palette.panel_bg),
            );
            x = x.saturating_add(separator_width);
        }
        let width = display_width(&segment.text);
        let style = if segment.accent {
            Style::default()
                .fg(panel_contrast_fg(palette))
                .bg(palette.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.overlay1).bg(palette.panel_bg)
        };
        put_text(buffer, x, area.y, width, &segment.text, style);
        x = x.saturating_add(width);
    }
}

fn tab_drop_indicator_x(
    hits: &ShellHitMap,
    tabs: &[&ClientShellTab],
    insert_index: usize,
) -> Option<u16> {
    let visible = hits
        .tabs
        .iter()
        .filter_map(|(rect, tab_id)| {
            tabs.iter()
                .position(|tab| tab.tab_id == *tab_id)
                .map(|index| (index, *rect))
        })
        .collect::<Vec<_>>();
    let (first_index, first_rect) = *visible.first()?;
    let (last_index, last_rect) = *visible.last()?;
    if insert_index == 0 {
        return Some(if first_index == 0 {
            first_rect.x
        } else {
            hits.tab_scroll_left.right()
        });
    }
    if let Some((_, rect)) = visible.iter().find(|(index, _)| *index == insert_index) {
        return Some(rect.x.saturating_sub(1));
    }
    if insert_index >= tabs.len() {
        return Some(if last_index + 1 >= tabs.len() {
            last_rect.right()
        } else {
            hits.tab_scroll_right.x.saturating_sub(1)
        });
    }
    None
}

fn centered_tab_scroll(focused: usize, widths: &[u16], available: u16) -> usize {
    let mut best = focused;
    let mut best_distance = u16::MAX;
    for start in 0..=focused {
        let before = widths
            .iter()
            .copied()
            .enumerate()
            .skip(start)
            .take(focused.saturating_sub(start))
            .fold(0u16, |width, (_, tab)| width.saturating_add(tab + 1));
        if before >= available {
            continue;
        }
        let focused_width = widths[focused].min(available.saturating_sub(before));
        let center = before.saturating_mul(2).saturating_add(focused_width);
        let distance = center.abs_diff(available);
        if distance <= best_distance {
            best_distance = distance;
            best = start;
        }
    }
    best
}

fn max_tab_scroll(widths: &[u16], available: u16) -> usize {
    let Some((&last, preceding)) = widths.split_last() else {
        return 0;
    };
    let mut start = preceding.len();
    let mut used = u32::from(last);
    // Keep the longest fully visible suffix, not merely a sliver of the last tab.
    // An oversized last tab must still be reachable at the start of the strip.
    for width in preceding.iter().rev() {
        let required = used + 1 + u32::from(*width);
        if required > u32::from(available) {
            break;
        }
        used = required;
        start -= 1;
    }
    start
}

// A card's content: the configured token of the tab's focused agent
// (any agent of the tab when none is focused), or the status word when the token is unset. Only
// the agents of one tab are scanned, and only for card tabs
fn tab_detail(
    tab: &ClientShellTab,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
) -> String {
    // A tab without an agent has nothing to report, so its card stays empty
    if !snapshot
        .agents
        .iter()
        .any(|agent| agent.tab_id == tab.tab_id && agent.agent.is_some())
    {
        return String::new();
    }
    let status = super::status_text(tab.agent_status).to_string();
    if config.tab_detail_token.is_empty() {
        return status;
    }
    let value = snapshot
        .agents
        .iter()
        .filter(|agent| agent.tab_id == tab.tab_id && agent.agent.is_some())
        .max_by_key(|agent| agent.focused)
        .and_then(|agent| {
            agent
                .tokens
                .iter()
                .find(|(name, _)| *name == config.tab_detail_token)
                .map(|(_, value)| value.as_str())
        });
    match value {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => status,
    }
}

// A card tab: the label on the rounded top border, the part before ` · ` (the project) in bold,
// two wrapped detail rows inside, and an empty bottom border. The border
// takes the accent on the focused tab
fn draw_card(
    buffer: &mut Buffer,
    rect: Rect,
    tab: &ClientShellTab,
    label: &str,
    detail: &str,
    config: &ClientShellConfig,
) {
    let palette = &config.palette;
    let (border, text) = if tab.focused {
        (palette.accent, palette.text)
    } else {
        (palette.overlay0, palette.overlay1)
    };
    let base = Style::default().bg(palette.panel_bg);
    let border_style = base.fg(border);
    let width = rect.width;
    let pad = config.tab_padding.min(width.saturating_sub(2) / 2);
    let inner = width.saturating_sub(2 + pad * 2);
    let y = rect.y;

    // Top border: ╭─ PROJECT · task ─╮
    let rule = |n: u16| "\u{2500}".repeat(n.into());
    put_text(
        buffer,
        rect.x,
        y,
        width,
        &format!("\u{256d}{}\u{256e}", rule(width.saturating_sub(2))),
        border_style,
    );
    let label = crate::ui::truncate_end(label, inner.saturating_sub(2).into());
    let (project, task) = label
        .split_once(" \u{b7} ")
        .map_or((label.as_str(), None), |(p, t)| (p, Some(t)));
    let mut x = rect.x + 1 + pad;
    if inner >= 3 {
        put_text(buffer, x, y, 1, " ", base);
        x += 1;
        let style = base.fg(text).add_modifier(Modifier::BOLD);
        put_text(buffer, x, y, display_width(project), project, style);
        x += display_width(project);
        if let Some(task) = task {
            let rest = format!(" \u{b7} {task}");
            put_text(buffer, x, y, display_width(&rest), &rest, base.fg(text));
            x += display_width(&rest);
        }
        put_text(buffer, x, y, 1, " ", base);
    }

    // Two content rows, without a duplicate status glyph.
    for row in 1..=2 {
        put_text(
            buffer,
            rect.x,
            y + row,
            width,
            &format!("│{:w$}│", "", w = width.saturating_sub(2).into()),
            border_style,
        );
    }
    ratatui::widgets::Widget::render(
        ratatui::widgets::Paragraph::new(detail)
            .style(base.fg(text))
            .wrap(ratatui::widgets::Wrap { trim: true }),
        Rect::new(rect.x + 1 + pad, y + 1, inner, 2),
        buffer,
    );

    // Bottom border
    put_text(
        buffer,
        rect.x,
        y + 3,
        width,
        &format!("\u{2570}{}\u{256f}", rule(width.saturating_sub(2))),
        border_style,
    );
}

fn tab_label(tab: &ClientShellTab) -> String {
    if tab.zoomed {
        format!("{} Z", tab.label)
    } else {
        tab.label.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::max_tab_scroll;

    #[test]
    fn trailing_scroll_limit_accounts_for_full_widths_and_separators() {
        for (widths, available, expected) in [
            (&[][..], 0, 0),
            (&[8, 13][..], 0, 1),
            (&[8, 13][..], 1, 1),
            (&[8, 13][..], 12, 1),
            (&[8, 13][..], 21, 1),
            (&[8, 13][..], 22, 0),
            (&[8, 13][..], 30, 0),
            (&[8, u16::MAX][..], u16::MAX, 1),
        ] {
            assert_eq!(
                max_tab_scroll(widths, available),
                expected,
                "widths={widths:?}, available={available}"
            );
        }
    }
}
