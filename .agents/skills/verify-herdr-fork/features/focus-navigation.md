# Focus and navigation

A focused card has a different border color, not a different fill. Users can select cards through the CLI, prefix-key navigation, or a click on either content row.

## Sub-features

- `focus-cli`: `tab focus` selects the requested tab.
- `focus-keyboard`: `Ctrl+B`, then `n` selects the next tab.
- `focus-mouse`: clicks on each content row select the corresponding card.
- `focus-style`: focused/inactive borders differ; card backgrounds stay equal.

## How to get to it (user POV)

Use `herdr tab focus`, press the default next-tab binding, or click a card. The private config uses the default keybindings and explicitly enables mouse capture.

## Driving it with verify-herdr

Preconditions: a built fork and a fresh instance; no metadata recipe needs to run first.

- **Fresh instance.** `"$V" run focus-navigation` creates its own `CFG · verification` and `HA · second` tabs.
- **CLI.** It focuses the returned CFG tab ID. `cli-focus.json` independently confirms the selected tab.
- **Style.** `cli-focus.ansi` and `cli-focus.styles.json` must show different border foregrounds and one shared background across all four card rows.
- **Keyboard.** The helper sends actual `C-b`, `n` keystrokes through tmux into Herdr. `keyboard-focus.json` must show HA focused.
- **Mouse.** It locates CFG from the visible top-border label, sends SGR mouse press/release on content rows `2` and `3`, and checks the focus readback after each click. `mouse-row-2.*` and `mouse-row-3.*` prove both paths.
- **Cleanup.** The helper removes only its own instance. `actions.jsonl` retains the keyboard/mouse commands and focus readbacks.

## Gotchas

- Mouse coordinates are derived from the visible label, not hard-coded card positions.
- A click that leaves the already-focused tab selected proves nothing; each mouse check starts with HA focused.
- The private tmux server has no user bindings or options. Running this inside the user's tmux session would invalidate isolation.
