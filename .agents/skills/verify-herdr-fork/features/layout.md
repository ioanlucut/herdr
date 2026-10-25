# Layout and defaults

Cards work at the top or bottom. Very short terminals fall back to line tabs, and removing all fork-specific keys restores upstream line defaults.

## Sub-features

- `layout-top`: cards use the first four terminal rows.
- `layout-bottom`: cards use the last four terminal rows after config reload.
- `layout-short`: a terminal only four rows high uses line tabs, preserving pane space.
- `layout-defaults`: omitting fork keys selects upstream line style.

## How to get to it (user POV)

Set `ui.tab_bar_position` to `top` or `bottom`, then run `herdr server reload-config`. Resize the terminal window. Remove `tab_style`, `tab_padding`, and `tab_detail_token` to use upstream defaults.

## Driving it with verify-herdr

Preconditions: a built fork; no tabs/config from another feature. The helper writes only its private test config.

- **Fresh instance.** `"$V" run layout` creates its own labelled card and goal.
- **Top.** `top-card.*` captures the initial top card.
- **Bottom.** Rewrite the private config with `tab_bar_position = "bottom"` and invoke the public reload command. `bottom-card.txt` must put the top border four rows from the end.
- **Short terminal.** Reload top position, then resize the owned tmux window to four rows. `short-screen.*` must contain no rounded card border.
- **Defaults.** Restore `170×32`, remove every fork-only key, and reload. `default-line.*` must contain the tab label but no card border.
- **Proof.** `actions.jsonl` retains each config and reload/resize command; captures retain visible and API state. Cleanup removes only the private config and process tree.

## Gotchas

- This uses the production reload path, not a test endpoint.
- Geometry is the outer terminal size, supplied by the private tmux window.
- Tiny-screen fallback verifies minimum height, not horizontal overflow or scrolling. Those remain separate Rust-test coverage.
- A later doctor call permits line style when the recorded private config intentionally omits card keys.
