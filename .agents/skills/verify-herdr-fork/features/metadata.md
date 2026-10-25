# Pane metadata

Cards follow public pane-token updates, drop cleared goals, select the focused agent of a split tab, and leave shell-only cards empty.

## Sub-features

- `metadata-set-update`: new token values replace previous text.
- `metadata-clear`: removing `tldr` removes the old goal and reveals the status word.
- `metadata-split`: the focused split agent supplies the card's goal.
- `metadata-shell`: a shell-only card leaves both content rows empty.

## How to get to it (user POV)

Integrations use `herdr pane report-agent` and `herdr pane report-metadata`. Users split a pane, focus a split, or open a plain shell tab. The fork reads the token configured by `ui.tab_detail_token`.

## Driving it with verify-herdr

Preconditions: a built fork; the helper creates every tab and lifecycle/token fixture itself.

- **Fresh instance.** `"$V" run metadata` starts from empty state.
- **Set.** Report `Initial login goal`. `initial-token.*` proves both API storage and real card text.
- **Update.** Report `Updated login goal`. `updated-token.*` must show it without the old text.
- **Clear.** Run `pane report-metadata --clear-token tldr`. `cleared-token.*` must show `working`, not the previous goal.
- **Split.** Run `pane split --direction right --focus`, read the returned pane ID, and report `Focused split goal`. `focused-split.*` must show that goal on the same tab's card.
- **Shell.** Open `SH · no agent` without reporting lifecycle state. `empty-shell.*` must have blank content cells beneath that label.
- **Proof.** Read `actions.jsonl` for the production report calls and `*.json` for token/focus readbacks. All artifacts survive cleanup.

## Gotchas

- The `pi` lifecycle label is a fixture through the public integration API; no Pi subprocess or model call is made.
- These checks verify the fork's token consumer. They do not prove the separate TLDR-producing plugin, transcript discovery, or cache invalidation.
- Never use personal pane IDs or plugin state to supply this feature's preconditions.
