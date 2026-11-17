# Tab link

Ctrl+click on a tab opens the http(s) link its agent reports in the pane token named by `tab_link_token`, the modifier that opens links inside panes.

## Sub-features

- `link-open`: Ctrl+click on a linked tab opens its link once and keeps focus where it was.
- `link-unlinked`: Ctrl+click on a tab without the token selects it and opens nothing.
- `link-plain`: a plain click on a linked tab selects it and opens nothing.

## How to get to it (user POV)

Set `[ui].tab_link_token = "issue_url"` with card tabs. An integration reports the link with `pane report-metadata --token issue_url=https://...`; the my-cfg tab-tldr plugin reports the tab's GitHub issue this way.

## Driving it with verify-herdr

- **Fresh instance.** `"$V" run tab-link` launches its own TUI with a private `bin/open` first on `PATH`, which appends each URL to `evidence/opened.txt` instead of opening a browser.
- **Fixtures.** Tab `CFG · verification` gets `issue_url=https://github.com/me/app/issues/24`; tab `HA · no link` gets no link and is focused.
- **Drive.** SGR mouse down/up on content row one at the visible label's column: modifier `16` (Ctrl) on `CFG`, Ctrl on `HA`, then modifier `0` on `CFG`.
- **Observe.** `opened.txt` holds the issue URL exactly once; focus stays on `HA` after the first click, then moves to `HA` and `CFG` through the API read.

## Gotchas

- Only http and https links open; other schemes fall back to selecting the tab (Rust test in `src/client/shell/tests/tab_rows.rs`).
- macOS terminals may turn Ctrl+click into a right click before Herdr sees it; this run sends the SGR event directly, so it does not cover the outer terminal.
