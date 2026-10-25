# Card content

A card puts its label on the top border and wraps its goal over two content rows without adding a second status glyph.

## Sub-features

- `card-frame`: top border, two content rows, bottom border.
- `card-wrap`: long goal wraps at a word boundary onto row two.
- `card-no-glyph`: neither content row adds a lifecycle glyph.
- `card-bold`: project abbreviation is bold; task text is not.

## How to get to it (user POV)

Enable `[ui].tab_style = "card"`, `tab_padding = 1`, and `tab_detail_token = "tldr"` in config. A user/integration names a tab and reports its pane's lifecycle and `tldr` token with Herdr's public CLI. No personal plugin is needed.

## Driving it with verify-herdr

Preconditions: a built fork candidate, tmux, Python, and an otherwise empty run directory created by the helper.

- **Fresh instance.** `"$V" run card-content` launches its own TUI.
- **Label.** The helper runs `tab create --label "CFG · verification"`, reading workspace/tab/pane IDs from API responses.
- **Content.** It runs `pane report-agent` and `pane report-metadata --token tldr=...` for that pane. The exact goal is `Build a bigger two-line herdr tab with hover details and status, then fork`.
- **Observe.** `wrapped.txt` shows `Build a bigger two-line herdr tab with hover details and` on content row one and `status, then fork` on row two. The bottom border is terminal row four. Neither content row contains `×`, `◐`, `✓`, or `○`.
- **Style.** Captured ANSI proves `CFG` is bold and `verification` is not. Inspect `wrapped.html` as a visual check.
- **Side effect.** `actions.jsonl` and `wrapped.json` show the report commands and the stored pane token. `result.json` records `PASS`; cleanup keeps these files.

## Gotchas

- The lifecycle and token are public integration fixtures. This does not verify agent detection or AI summarization.
- Changing terminal width changes wrapping; the harness fixes it at `170×32`.
- A short goal legitimately leaves the second row blank; the recipe deliberately uses a long goal.
- Text beyond two content rows is clipped; clipping and Unicode edge cases also have Rust tests in `src/client/shell/tests/tab_rows.rs`.
