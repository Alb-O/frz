# Kitty Image Artifact Bug

## Current Symptom

When moving selection between images, the **results pane** still gets corrupted text on the previously imaged row (e.g. `t split: [6;29;13` replacing the scrollbar/dividers). Repro continues to fail via:

```bash
kitty --dump-commands=yes sh -c "cargo test -p kitty-test-harness -- --ignored" | grep "FAILED"
```

Failing test: `crates/kitty-tests/tests/desktop_preview_artifact.rs` (pattern `t split:` in cleaned output).

## What We Tried (and results)

- Removed Kitty cursor jumps (`\x1b[{right}C\x1b[{down}B`), added color resets, and later rewrote Kitty placeholder rendering to per-cell placeholders instead of one big first-cell blob. **Artifact still present.**
- Rendered preview before results to let results overwrite any escape fallout. **No change.**
- Attempted to disable Kitty by sanitizing the picker and short-circuiting when protocol is Kitty (debug prints show protocol resolves to Halfblocks), yet the `t split:` artifact still appears. **So the string is not coming from live Kitty rendering.**
- Added debug print of screen contents in the failing test to capture the exact corruption; the string is rendered in the results table, not in the preview pane.

## New Findings

- `vendor/ratatui-image/src/picker/cap_parser.rs` contains a stray debug print: `println!("t split: {}", self.data);`. This can emit `t split:` text during capability probing, polluting the alternate screen. The artifact likely comes from this debug println, not from image rendering.
- The artifact persists even when the picker chooses Halfblocks (kitty disabled), reinforcing that the leakage is from capability probing stdout rather than Kitty protocol drawing.

## Likely Root Cause

A leftover debug `println!` in `ratatui-image` capability parser writes `t split:` to stdout during protocol detection. That output lands in the TUI buffer and shows up in the results pane on the previously selected row.

## Next Actions

- Remove the `println!("t split: {}", ...)` debug line in `vendor/ratatui-image/src/picker/cap_parser.rs` and re-run the ignored kitty test.
- If still failing, re-check for any other stdout/stderr writes during capability probing or image rendering.

## Relevant Paths

- `crates/tui/src/components/preview/render.rs`
- `crates/tui/src/components/preview/image.rs`
- `crates/tui/src/app/render/mod.rs`
- `vendor/ratatui-image/src/protocol/kitty.rs`
- `vendor/ratatui-image/src/picker/cap_parser.rs` (suspect debug println)
- Test: `crates/kitty-tests/tests/desktop_preview_artifact.rs`
