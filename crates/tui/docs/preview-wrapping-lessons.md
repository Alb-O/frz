# Preview Wrapping: Lessons Learned

## Context
- The preview pane renders `bat`-highlighted output, then wraps lines to the visible width while preserving the line-number gutter.
- Wrapping happens in-process (Ratatui) after `bat` emits ANSI-styled spans; the scrollbar state and mouse dragging rely on the wrapped line count.

## Key Challenges
- **Gutter detection**: Bat sometimes emits gutters without an explicit `│`, so we had to detect gutters by leading digits/spaces instead of hardcoding separators.
- **Indent preservation**: Wrapped continuations must keep both the gutter width and the original code indentation; naïve wrapping pushed text into the gutter or stripped indent.
- **Span-safe slicing**: Wrap calculations must respect Unicode width, avoid splitting multi-byte characters, and keep styles aligned to the correct text fragments.
- **Scrollbar alignment**: The scrollbar needs the wrapped length, not the raw line count, otherwise scrolling and drag math drift.
- **Snapshot fidelity**: Tests must render via the real `bat` pipeline to catch gutter/indent regressions—handcrafted spans hid real-world issues.

## Implemented Techniques
- **Gutter-aware wrap**: Split spans into `(gutter, body)` by scanning for `│` or leading digits/spaces; wrap only the body while carrying gutter width and leading indentation onto continuations.
- **Width-aware chunking**: Compute wrap breaks using Unicode width per grapheme, splitting spans at width boundaries without breaking character boundaries.
- **Continuation indentation**: Continuation lines reapply gutter padding plus detected leading indent to keep code visually aligned.
- **State alignment**: Rebuild wrapped lines before rendering, then use wrapped length for scrollbar content/position.
- **Snapshot validation**: Ratatui `TestBackend` + `highlight_with_bat` snapshots verify gutters aren’t polluted and continuations stay indented; dynamic gutter-width detection keeps assertions resilient to theme/layout changes.

## Workarounds & Trade-offs
- **Detect gutters heuristically** when `│` is absent; this favors robustness across bat themes but assumes gutters are all-leading whitespace/digits.
- **Render without Paragraph’s trim** because pre-wrapped lines already include intentional leading spaces.
- **Dynamic gutter width in tests** to avoid brittleness across bat styles or terminal widths.

## Risks & Future Improvements
- **Heuristic gutter split** could misclassify odd files (e.g., lines starting with digits). Consider tagging gutter spans earlier in the pipeline.
- **Indent heuristic** treats tabs as 4 spaces; configurable tab width would be more accurate.
- **Snapshot drift**: Intentional UI tweaks require `cargo insta review`; consider additional unit tests on wrapper geometry to localize failures.
- **Performance**: Re-wrapping on every draw is acceptable for current sizes but could be cached per width/content hash if needed.
