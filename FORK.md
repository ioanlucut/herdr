# This is a personal fork

`ioanlucut/herdr` is [herdrdev/herdr](https://github.com/herdrdev/herdr) plus card tabs. Do not open pull requests from it to upstream. Upstream accepts no unsolicited pull requests (`CONTRIBUTING.md`).

## What is different

Branch `feat/tab-bar-rows` is upstream's latest release tag plus these commits:

| Key (`[ui]`) | Default | Effect |
|---|---|---|
| `tab_style` | `"line"` | `"card"` draws each tab as a four-row bordered card. The label is on the top border, and the part before ` · ` is bold. The status glyph and the `tab_detail_token` value are inside. Only the border marks the focused card. |
| `tab_padding` | `2` | Cells on each side of a tab label, for both styles. |
| `tab_detail_token` | `""` | The pane metadata token shown inside a card, read from the tab's focused agent. |

The defaults keep upstream behavior. The code is in `src/client/shell/tabs.rs` and `src/client/shell/config.rs`, and the config is in `src/config/model.rs`. The tests are in `src/client/shell/tests/tab_rows.rs`. The docs are in `docs/next/website/src/content/docs/configuration.mdx` ("Card tabs") and `config-reference.json`.

## How it is kept current

The owner's config repo runs everything: `~/WORK/dev/my-cfg`, `just herdr-fork` (`scripts/herdr-fork.sh`). It fetches upstream's tags, rebases this branch onto the latest release with `git rebase --onto <latest> <old tag>`, runs fmt, clippy, the shell and config tests, and the config-reference check, and builds a release binary. It then links the binary over `/opt/homebrew/bin/herdr` and force-pushes this branch. `just verify` in that repo fails until this runs after each upstream release. The reasons are in `my-cfg/docs/adr/0001-fork-herdr-for-card-tabs.md`.

Build prerequisites: the Rust toolchain pinned in `rust-toolchain.toml` (through `rustup`) and Zig `0.16` (`ZIG=/opt/homebrew/opt/zig@0.16/bin/zig`).

The upstream test suite has 17 failures on the owner's machine even at the untouched `v0.9.5` tag: plugin-path and agent-manifest tests that depend on the environment. A fork change must not add a new failure. Compare it with a run at the base tag.
