# chess-binpack-utils

General-purpose Rust CLI for working with chess binpack data.

## Install As A Tool

```bash
cargo install --git https://github.com/Disservin/chess-binpack-utils.git
```

This installs the `chess-binpack-utils` executable from GitHub into Cargo's bin directory.

To install the local checkout instead, use:

```bash
cargo install --path .
```

## Commands

The main CLI exposes four operations:

- `convert`
- `unique`
- `inspect`
- `benchmark`

After that, you can run commands directly:

```bash
chess-binpack-utils convert --input test/ep1.binpack --output out.viri
chess-binpack-utils unique --input test/ep1.binpack
chess-binpack-utils inspect --input test/ep1.binpack
chess-binpack-utils benchmark --input test/ep1.binpack
```

Formats are inferred from file extensions when possible:

- `.vf`, `.viri`, `.viriformat` -> `viriformat`
- `.sf`, `.sfbinpack`, `.binpack` -> `sfbinpack`
- `.bf`, `.bullet`, `.bulletformat` -> `bulletformat`
- `.txt`, `.bulletplain` -> `bulletplain`

Format names:

- `bulletformat`: Bullet's binary packed chess format
- `bulletplain`: Bullet's plain-text chess format, where each line is `<FEN> | <score> | <result>`

## What It Does

- Converts between supported formats
- Counts unique positions in game-oriented formats
- Benchmarks read throughput across supported formats

### Convert

```bash
chess-binpack-utils convert --input <INPUT> --output <OUTPUT>
```

Supported conversions:

- `sfbinpack` -> `viriformat`
- `viriformat` -> `sfbinpack`
- `sfbinpack` -> `bulletformat`
- `viriformat` -> `bulletformat`
- `bulletplain` -> `bulletformat`

To stop after a fixed number of entries, pass `--limit <N>`.
For game-based formats, the limit counts positions/training entries and may truncate the last game.
For `bulletplain -> bulletformat`, the limit counts non-empty input lines.

You can still override inference explicitly:

```bash
chess-binpack-utils convert --from <sfbinpack|viriformat|bulletformat|bulletplain> --to <sfbinpack|viriformat|bulletformat> --input <INPUT> --output <OUTPUT>
```

Example with inferred formats:

```bash
chess-binpack-utils convert \
  --input test/ep1.binpack \
  --output out.viri
```

Example with a limit:

```bash
chess-binpack-utils convert \
  --input test/ep1.binpack \
  --output out.viri \
  --limit 1000
```

Example with explicit formats:

```bash
chess-binpack-utils convert \
  --from sfbinpack \
  --to viriformat \
  --input test/ep1.binpack \
  --output out.viri
```

Reverse conversion:

```bash
chess-binpack-utils convert \
  --input out.viri \
  --output roundtrip.binpack
```

Bullet plain-text to bulletformat:

```bash
chess-binpack-utils convert \
  --from bulletplain \
  --to bulletformat \
  --input positions.txt \
  --output positions.bf
```

### Unique

```bash
chess-binpack-utils unique --input <INPUT>
```

This prints the number of unique positions found in the input.

Supported backends:

- `sfbinpack`
- `viriformat`

The backend is inferred from the input file extension when possible:

- `.vf`, `.viri`, `.viriformat` -> `viriformat`
- `.sf`, `.sfbinpack`, `.binpack` -> `sfbinpack`

You can also set it explicitly:

```bash
chess-binpack-utils unique --backend <sfbinpack|viriformat> --input <INPUT>
```

To stop after a fixed number of positions, pass `--limit <N>`:

```bash
chess-binpack-utils unique --input test/ep1.binpack --limit 1000
```

### Inspect

```bash
chess-binpack-utils inspect --input <INPUT>
```

This prints the input entries to stdout.

Supported formats:

- `sfbinpack`
- `viriformat`
- `bulletformat`
- `bulletplain`

You can override inference explicitly:

```bash
chess-binpack-utils inspect --format <sfbinpack|viriformat|bulletformat|bulletplain> --input <INPUT>
```

To stop after a fixed number of entries, pass `--limit <N>`:

```bash
chess-binpack-utils inspect --input test/ep1.binpack --limit 10
```

### Benchmark

```bash
chess-binpack-utils benchmark --input <INPUT>
```

This scans the input and reports throughput.

Supported formats:

- `sfbinpack`
- `viriformat`
- `bulletformat`
- `bulletplain`

You can override inference explicitly:

```bash
chess-binpack-utils benchmark --format <sfbinpack|viriformat|bulletformat|bulletplain> --input <INPUT>
```

Examples:

```bash
chess-binpack-utils benchmark --input test/ep1.binpack
chess-binpack-utils benchmark --input out.viri
chess-binpack-utils benchmark --input positions.bf
```

## Limitations

- Converting a format to itself is rejected
- `bulletformat` -> `sfbinpack` and `bulletformat` -> `viriformat` are rejected because `bulletformat` stores standalone positions, not move sequences
- `viriformat` input using Chess960-style castling rights is not supported when writing `viriformat` output
- `viriformat` game outcomes must be representable as win, draw, or loss in `sfbinpack`

For `bulletplain`, each input line is:

```text
<FEN> | <score> | <result>
```

- `score` is white-relative centipawns
- `result` is white-relative and must be `1.0`, `0.5`, or `0.0`

## Test

```bash
cargo test
```

The test suite includes round-trip checks for both formats, unique-position tests, and a fixture-based conversion test using `test/ep1.binpack`.

## License

This project is licensed under the GNU General Public License v3.0. See `LICENSE`.
