# Ideal Output Samples

This folder contains small Mermaid *flowchart* inputs (`.mmd`) and corresponding **idealized** Unicode ASCII-art outputs (`.unicode.ideal.txt`).

These are intentionally aspirational: the goal is to have stable targets for layout/rendering discussions and to diff against the current renderer output.

## Compare against current output

To generate the current output for a sample:

```bash
FIGUREHEAD_LOG_LEVEL=off cargo run -p figurehead-cli -- convert -i docs/ideal-output/01_basic_lr.mmd --style unicode
```

Then diff it against the ideal file:

```bash
diff -u docs/ideal-output/01_basic_lr.unicode.ideal.txt <(FIGUREHEAD_LOG_LEVEL=off cargo run -p figurehead-cli -- convert -i docs/ideal-output/01_basic_lr.mmd --style unicode)
```

