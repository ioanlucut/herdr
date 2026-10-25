---
name: verify-herdr-fork
description: Verify the ioanlucut Herdr fork's real TUI after changes to card content, focus/navigation, pane metadata, or layout/defaults. Each feature runs independently in a new private tmux server with its own home, config, state, and evidence; no live Herdr session or personal plugin is used.
---

# Verify the Herdr fork

Primary surface: the actual terminal UI. Secondary surface: the public Herdr CLI used by users and integrations. Read [the feature index](features/README.md), then select one feature. Each recipe starts from empty state; no preceding recipe is required.

This harness targets Unix with `tmux`, Python `3.10+`, and the fork's pinned Rust/Zig toolchain. It does not verify Windows, remote clients, real coding-agent detection, AI-generated TLDRs, or the personal `my-cfg` plugins. Lifecycle and token inputs use Herdr's production `pane report-agent` and `pane report-metadata` commands, the same external integration boundary used by hooks and plugins. They are explicit fixtures, not proof that an AI agent ran.

## Launch

From this checkout:

```bash
V="$PWD/.agents/skills/verify-herdr-fork/verify-herdr"
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
export ZIG=/opt/homebrew/opt/zig@0.16/bin/zig
"$V" build
"$V" launch card-content
```

On other Unix systems, use the Rust toolchain in `rust-toolchain.toml` and point `ZIG` at Zig `0.16`. `build` runs `cargo build --release --locked`. Default candidate: this checkout's `target/release/herdr`. If using `CARGO_TARGET_DIR`, also set `HERDR_VERIFY_BIN` to that build's absolute path.

`launch` prints a unique run directory as its last line, such as `/private/var/tmp/hfv-abc123`. Copy that actual value into `R`; never guess IDs or reuse another run's directory. Readiness means the private TUI visibly draws cards, the named server answers, at least one pane exists, and no plugins are loaded.

Every launch uses a separate tmux server/socket, a new `HOME`, XDG config/state/cache roots, a minimal commented test config, and `/bin/sh` without user startup files. Inherited `HERDR_*`, tmux options, credentials, and personal plugins are excluded. The config permits nesting explicitly, skips onboarding, and disables update network checks. No host Herdr pane is created or controlled.

Teardown: `"$V" cleanup "$R"`, including after a failed manual launch/drive.

## Doctor

```bash
"$V" doctor "$R"
```

Read-only: verifies the exact binary hash recorded at launch, the owned tmux session, the private Herdr session directory, usable panes, absence of plugins, and the expected card surface when geometry/config permits it. It changes no app state. Local evidence gets a diagnostic log entry.

The run manifest records candidate path/hash and checkout revision. A matching version number alone does not establish source identity; build from the checkout under test before launch. A changed binary fails doctor: clean up that run and start another.

## Drive

The shortest complete proof runs one isolated feature and always cleans up, on success or failure:

```bash
"$V" run card-content
"$V" run focus-navigation
"$V" run metadata
"$V" run layout
```

These are independent alternatives, not ordered steps. For a manual inspection window, use `launch FEATURE`, then:

```bash
"$V" doctor "$R"
"$V" drive "$R"
"$V" capture "$R"
"$V" cleanup "$R"
```

Run `drive` once per fresh instance. It creates its own tabs, panes, and metadata through the public CLI. Navigation also sends real keyboard and SGR mouse events into the TUI's PTY. Stable handles come from API responses and visible labels, not fixed pane IDs or guessed screen coordinates.

## Evidence

The helper prints the exact `RUN_DIR/evidence/` path. After cleanup it still contains:

| Artifact | Proof |
|---|---|
| `../run.json` | feature ID, binary path/hash, checkout revision, owned run root |
| `actions.jsonl` | actual CLI commands, stdout/stderr/exit status, tmux actions, and config reload inputs |
| `before.*` and feature-named captures | before/after terminal text, ANSI styles, HTML preview, pane/token and tab/focus JSON readbacks |
| `*.styles.json` | decoded foreground/background/bold per character, derived from captured ANSI |
| `result.json` | `PASS` only after all assertions for the selected feature succeed |
| `failure.*` | failed-run screen and state when capture is possible |

Inspect the ANSI or HTML as well as the JSON. Card content must appear on the real screen; a token readback alone is insufficient. Bold and border/fill assertions read the TUI's ANSI output. Navigation proves both the input action and the resulting focus through a separate API read. Metadata proves the public report command, pane readback, and rendered result.

HTML is a preview, not a browser implementation of Herdr. Terminal text and ANSI are authoritative. Capture files are sensitive if a future journey puts sensitive text in panes; current fixtures contain no credentials.

## Cleanup

```bash
"$V" cleanup "$R"
test -s "$R/evidence/actions.jsonl"
test -s "$R/evidence/result.json"  # successful drives only
```

Stops only `probe` under the recorded private XDG root, then the tmux server addressed by this run's private socket. Confirms those sockets are gone and removes private home/config/state/cache/work directories. Preserves the manifest, logs, captures, and result. Repeating cleanup is safe. It never kills by process name or calls the user's default Herdr socket.

`run FEATURE` performs this cleanup in a `finally` block. For manual commands, cleanup remains the caller's responsibility after errors. A failed or skipped assertion remains a failure; never remove a precondition to obtain `PASS`.

## Helpers

`./.agents/skills/verify-herdr-fork/verify-herdr` is executable and ships with this skill:

- `build`: compile this checkout's release candidate.
- `launch FEATURE`: start a fresh isolated instance and print its run directory.
- `doctor RUN_DIR`: read-only ownership/readiness check.
- `drive RUN_DIR`: exercise that instance's selected feature.
- `capture RUN_DIR`: save a manual terminal/state capture.
- `cleanup RUN_DIR`: remove owned processes and scratch state, keep evidence.
- `run FEATURE`: launch, doctor, drive, capture, and guaranteed cleanup.

Use `/maintain-verification-skill` when fork behavior changes; update the feature map and executable assertions together.
