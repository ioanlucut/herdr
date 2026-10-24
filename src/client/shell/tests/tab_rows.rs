use super::*;
use crate::config::TabStyleConfig;

fn row_text(frame: &FrameData, row: u16) -> String {
    let width = usize::from(frame.width);
    let start = usize::from(row) * width;
    frame.cells[start..start + width]
        .iter()
        .map(|cell| cell.symbol.as_str())
        .collect()
}

fn cell(frame: &FrameData, x: u16, y: u16) -> &crate::protocol::CellData {
    &frame.cells[usize::from(y) * usize::from(frame.width) + usize::from(x)]
}

fn agent(tab_id: &str, pane_id: &str, focused: bool, tokens: &[(&str, &str)]) -> ClientShellAgent {
    ClientShellAgent {
        pane_id: pane_id.into(),
        workspace_id: "ws_1".into(),
        tab_id: tab_id.into(),
        name: None,
        display_agent: None,
        agent: Some("pi".into()),
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: AgentStatus::Working,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: tokens
            .iter()
            .map(|(name, value)| ((*name).into(), (*value).into()))
            .collect(),
        focused,
    }
}

// A new snapshot revision needs a pane surface of the same projection revision, or compose skips the frame
fn replace_snapshot(state: &mut ClientShellState, edit: impl FnOnce(&mut ClientShellSnapshot)) {
    let mut projected = state.snapshot.as_deref().expect("snapshot").clone();
    edit(&mut projected);
    projected.revision += 1;
    let mut pane_surface = surface();
    pane_surface.projection_revision = projected.revision;
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(pane_surface);
}

fn card_config(tab_detail_token: &str) -> ClientShellConfig {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.mobile_width_threshold = 0;
    config.tab_style = TabStyleConfig::Card;
    config.tab_padding = 1;
    config.tab_detail_token = tab_detail_token.into();
    config.status_indicators = crate::config::StatusIndicatorStyle::Symbols;
    config
}

fn card_state(tab_detail_token: &str) -> ClientShellState {
    let mut projected = snapshot();
    projected.tabs[0].label = "CFG \u{b7} fix login".into();
    projected.tabs[0].agent_status = AgentStatus::Working;
    projected.agents = vec![agent(
        "tab_1",
        "pane_1",
        true,
        &[("tldr", "Fix the login redirect")],
    )];
    let mut state = ClientShellState::new(card_config(tab_detail_token));
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    state
}

fn second_tab(projected: &mut ClientShellSnapshot) {
    projected.tabs.push(ClientShellTab {
        tab_id: "tab_2".into(),
        workspace_id: "ws_1".into(),
        number: 2,
        label: "HA \u{b7} garage".into(),
        custom_label: false,
        zoomed: false,
        focused: false,
        agent_status: AgentStatus::Blocked,
    });
}

#[test]
fn default_config_keeps_line_tabs_with_two_cells_of_padding() {
    let config = ClientShellConfig::from_config(&Config::default());
    assert_eq!(config.tab_style, TabStyleConfig::Line);
    assert_eq!(config.tab_padding, 2);
    assert!(config.tab_detail_token.is_empty());
    let layout = config.layout(106, 20, false, 1, 26);
    assert_eq!(layout.tab_bar.height, 1);
    assert_eq!(layout.pane_surface.y, 1);
}

#[test]
fn card_tab_bar_takes_three_rows_at_the_top_or_the_bottom() {
    let config = card_config("tldr");
    let layout = config.layout(106, 20, false, 1, 26);
    assert_eq!((layout.tab_bar.y, layout.tab_bar.height), (0, 3));
    assert_eq!((layout.pane_surface.y, layout.pane_surface.height), (3, 17));

    let mut bottom = card_config("tldr");
    bottom.tab_bar_position = crate::config::TabBarPositionConfig::Bottom;
    let layout = bottom.layout(106, 20, false, 1, 26);
    assert_eq!((layout.tab_bar.y, layout.tab_bar.height), (17, 3));
    assert_eq!(layout.pane_surface.height, 17);
}

#[test]
fn card_tab_bar_never_takes_the_last_pane_row() {
    let config = card_config("tldr");
    let layout = config.layout(106, 3, false, 1, 26);
    assert_eq!(layout.tab_bar.height, 2);
    assert_eq!(layout.pane_surface.height, 1);
}

#[test]
fn a_bar_too_short_for_cards_falls_back_to_line_tabs() {
    let mut state = card_state("tldr");
    let frame = state.compose(106, 3).expect("short card bar");
    let top = row_text(&frame, 0);
    assert!(!top.contains('\u{256d}'), "{top:?}");
    assert!(top.contains("CFG \u{b7} fix login"), "{top:?}");
}

#[test]
fn card_shows_the_label_on_the_border_and_the_tldr_inside() {
    let mut state = card_state("tldr");
    let frame = state.compose(106, 20).expect("card tab bar");
    let (top, mid, bottom) = (
        row_text(&frame, 0),
        row_text(&frame, 1),
        row_text(&frame, 2),
    );
    assert!(
        top.contains("\u{256d}\u{2500} CFG \u{b7} fix login \u{2500}"),
        "{top:?}"
    );
    assert!(
        mid.contains("\u{2502} \u{25d0} Fix the login redirect \u{2502}"),
        "{mid:?}"
    );
    assert!(
        bottom.contains('\u{2570}') && bottom.contains('\u{256f}'),
        "{bottom:?}"
    );
    assert!(row_text(&frame, 3).contains("LIVE"));
}

#[test]
fn card_puts_the_project_in_bold_and_nothing_else() {
    let mut state = card_state("tldr");
    let frame = state.compose(106, 20).expect("card tab bar");
    let top = row_text(&frame, 0);
    let bold = ratatui::style::Modifier::BOLD.bits();
    let at = |needle: &str| {
        top.find(needle)
            .map(|byte| u16::try_from(top[..byte].chars().count()).expect("narrow"))
    };
    let project = at("CFG").expect("project on the border");
    let task = at("fix").expect("task on the border");
    assert_ne!(cell(&frame, project, 0).modifier & bold, 0);
    assert_eq!(cell(&frame, task, 0).modifier & bold, 0);
}

#[test]
fn only_the_border_marks_the_focused_card() {
    let mut state = card_state("tldr");
    replace_snapshot(&mut state, second_tab);
    let frame = state.compose(106, 20).expect("two cards");
    let palette = &state.config.palette;
    let focused = state.hits.tabs[0].0;
    let other = state.hits.tabs[1].0;
    let accent = crate::protocol::color_to_u32(palette.accent);
    let grey = crate::protocol::color_to_u32(palette.overlay0);
    let panel = crate::protocol::color_to_u32(palette.panel_bg);
    assert_eq!(cell(&frame, focused.x, 1).fg, accent);
    assert_eq!(cell(&frame, other.x, 1).fg, grey);
    for rect in [focused, other] {
        for x in rect.x..rect.right() {
            for y in 0..3 {
                assert_eq!(cell(&frame, x, y).bg, panel, "fill at {x},{y}");
            }
        }
    }
}

#[test]
fn card_prefers_the_focused_agent_of_a_split_tab() {
    let mut state = card_state("tldr");
    replace_snapshot(&mut state, |projected| {
        projected.agents = vec![
            agent("tab_1", "pane_2", false, &[("tldr", "Background reviewer")]),
            agent("tab_1", "pane_1", true, &[("tldr", "Focused goal")]),
        ];
    });
    let frame = state.compose(106, 20).expect("split tab");
    let mid = row_text(&frame, 1);
    assert!(mid.contains("Focused goal"), "{mid:?}");
    assert!(!mid.contains("Background reviewer"));
}

#[test]
fn card_without_a_token_shows_the_status_word() {
    for token in ["", "missing"] {
        let mut state = card_state(token);
        let frame = state.compose(106, 20).expect("status-only card");
        let mid = row_text(&frame, 1);
        assert!(mid.contains("\u{25d0} working"), "{token:?}: {mid:?}");
    }
}

#[test]
fn card_without_an_agent_stays_empty() {
    let mut state = card_state("tldr");
    replace_snapshot(&mut state, |projected| {
        projected.agents.clear();
        projected.tabs[0].label = "zsh".into();
    });
    let frame = state.compose(106, 20).expect("shell card");
    let rect = state.hits.tabs[0].0;
    let mid: String = row_text(&frame, 1)
        .chars()
        .skip(usize::from(rect.x) + 1)
        .take(usize::from(rect.width) - 2)
        .collect();
    assert!(mid.trim().is_empty(), "{mid:?}");
}

#[test]
fn a_long_tldr_is_capped_and_cut_inside_the_card() {
    let mut state = card_state("tldr");
    replace_snapshot(&mut state, |projected| {
        projected.agents = vec![agent(
            "tab_1",
            "pane_1",
            true,
            &[("tldr", &"word ".repeat(40))],
        )];
    });
    let frame = state.compose(200, 20).expect("long detail");
    let rect = state.hits.tabs[0].0;
    // Text cap plus two borders and the padding on each side
    assert_eq!(rect.width, 60 + 2 + 2);
    let mid: String = row_text(&frame, 1)
        .chars()
        .skip(usize::from(rect.x))
        .take(usize::from(rect.width))
        .collect();
    assert!(mid.trim_end().ends_with("\u{2026} \u{2502}"), "{mid:?}");
}

#[test]
fn a_click_anywhere_on_a_card_selects_the_tab() {
    let mut state = card_state("tldr");
    replace_snapshot(&mut state, second_tab);
    state.compose(106, 20).expect("two cards");
    let (rect, tab_id) = state.hits.tabs[1].clone();
    assert_eq!(tab_id, "tab_2");
    assert_eq!(rect.height, 3);
    let click = |kind| {
        RawInputEvent::Mouse(MouseEvent {
            kind,
            column: rect.x + 1,
            row: rect.y + 2,
            modifiers: KeyModifiers::empty(),
        })
    };
    let outcome = state.handle_raw_events(vec![
        click(MouseEventKind::Down(MouseButton::Left)),
        click(MouseEventKind::Up(MouseButton::Left)),
    ]);
    assert!(
        outcome.actions.iter().any(|action| matches!(
            action,
            ClientShellAction::Endpoint { request, .. }
                if matches!(&request.method, crate::api::schema::Method::TabFocus(target) if target.tab_id == "tab_2")
        )),
        "{:?}",
        outcome.actions
    );
}

#[test]
fn controls_sit_on_the_middle_card_row() {
    let mut state = card_state("tldr");
    state.compose(106, 20).expect("card tab bar");
    assert_eq!(state.hits.new_tab.y, 1);
}

#[test]
fn tab_padding_widens_line_tabs() {
    let mut state = ClientShellState::new({
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.mobile_width_threshold = 0;
        config
    });
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 20).expect("default padding");
    assert_eq!(state.hits.tabs[0].0.width, 8);
    state.config.tab_padding = 6;
    state.compose(106, 20).expect("wide padding");
    assert_eq!(state.hits.tabs[0].0.width, 1 + 12);
}
