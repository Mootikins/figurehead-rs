# Ideal vs Current: Notable Gaps

This is a quick summary of what differs between the current renderer output and the idealized targets in this folder.

## Output hygiene

- **Leading left padding**: current outputs tend to have a leading space column before the first visible glyph.
- **Missing trailing newline**: current outputs end without a newline (`\n`), which makes diffs noisier.

## Layout / routing

- `02_decision_join_td`
  - **No “join” routing**: edges `C --> E` and `D --> E` do not merge cleanly; the current output produces a broken horizontal segment.
  - **Branching junction rendering**: the split from the decision node doesn’t render a clear junction glyph (e.g., `┴`) at the split point.
  - **Vertical spacing**: start→decision uses several redundant vertical segments; ideal target is more compact.

- `05_fanin_lr`
  - **Fan-in junction preference**: current output routes into `C` via separate stubs/corners rather than a distinct junction glyph (`├`/`┤`/`┼`). The ideal target prefers an explicit merge point.

## Feature gaps

- `03_subgraph_cluster_lr`
  - **Subgraph rendering**: `subgraph ... end` is currently parsed but not preserved/rendered as a cluster box/title. Ideal target shows a bounding box with the subgraph title and routing through the boundary.

## Node text shaping

- `04_wrapped_label_lr`
  - **No label wrapping**: long labels expand node width instead of wrapping to multiple lines with increased node height.

## Edge styling polish

- `06_edge_styles_lr`
  - **Cylinder connection clearance**: edge-to-cylinder contact is tight (arrowhead touches border). Ideal target leaves a 1-cell gap before the cylinder border.

## Additional coverage

- `07_shapes_gallery_lr`
  - Mostly a **style regression guard** across supported shapes; ideal target primarily differs in output hygiene (no leading pad, newline).

- `08_invisible_edges_td`
  - **Invisible-edge placement**: current layout tends to place the “spacer” node in the same rank as visible nodes in a surprising way; ideal target prefers treating `~~~` as a layout-only constraint without forcing same-rank placement.
