# Herdr fork verification map

Every feature below is independently runnable from an empty data directory. The harness allocates a fresh private tmux server, home, config/state roots, shell, and Herdr session per invocation. Features never borrow fixture IDs, metadata, processes, or evidence from each other.

Build once using [Launch](../SKILL.md#launch). From the checkout root, set `V="$PWD/.agents/skills/verify-herdr-fork/verify-herdr"`. Then run any one recipe. All test inputs are local; no agent account, paid request, network service, personal plugin, or existing Herdr session is required.

| Feature | Independent recipe | Covers |
|---|---|---|
| [Card content](card-content.md) | `"$V" run card-content` | four-row frame, two wrapped goal rows, no duplicate glyph, bold project only |
| [Focus and navigation](focus-navigation.md) | `"$V" run focus-navigation` | CLI focus, prefix keyboard navigation, mouse on both content rows, border focus without background fill |
| [Pane metadata](metadata.md) | `"$V" run metadata` | set/update/clear, status fallback, focused split agent, empty shell card |
| [Layout and defaults](layout.md) | `"$V" run layout` | top/bottom bars, narrow-height fallback, upstream line defaults |

Read the matching file before driving. `run` captures actions and terminal/API state, evaluates the feature's assertions, and cleans up even on failure. Report the feature ID and printed evidence directory. A successful feature does not verify another feature or an excluded path.

Scope: these are the fork's user-facing additions and adjacent regression paths, not an exhaustive verification of upstream Herdr. Real-agent detection, AI summaries, remote clients, Windows, and personal plugins remain outside this map.
