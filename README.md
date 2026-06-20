# chess-binpack-utils

Small Rust CLI for converting chess training data between `sfbinpack`, `viriformat`, and `bulletformat`.

## What It Does

- Reads game-oriented training data from one backend
- Streams it into the other backend
- Preserves move sequence, evaluation score, ply, initial FEN, and game result

Supported conversions:

- `sfbinpack` -> `viriformat`
- `viriformat` -> `sfbinpack`
- `sfbinpack` -> `bulletformat`
- `viriformat` -> `bulletformat`
- `bulletplain` -> `bulletformat`

## Limitations

- Converting a format to itself is rejected
- `bulletformat` -> `sfbinpack` and `bulletformat` -> `viriformat` are rejected because `bulletformat` stores standalone positions, not move sequences
- `viriformat` input using Chess960-style castling rights is not supported when writing `viriformat` output
- `viriformat` game outcomes must be representable as win, draw, or loss in `sfbinpack`

## Build

```bash
cargo build
```

Format names:

- `bulletformat`: Bullet's binary packed chess format
- `bulletplain`: Bullet's plain-text chess format, where each line is `<FEN> | <score> | <result>`

## Usage

```bash
cargo run -- convert --input <INPUT> --output <OUTPUT>
```

To stop after a fixed number of entries, pass `--limit <N>`.
For game-based formats, the limit counts positions/training entries and may truncate the last game.
For `bulletplain -> bulletformat`, the limit counts non-empty input lines.

Formats are inferred from file extensions when possible:

- `.vf`, `.viri`, `.viriformat` -> `viriformat`
- `.sf`, `.sfbinpack`, `.binpack` -> `sfbinpack`
- `.bf`, `.bullet`, `.bulletformat` -> `bulletformat`
- `.txt`, `.bulletplain` -> `bulletplain`

You can still override inference explicitly:

```bash
cargo run -- convert --from <sfbinpack|viriformat|bulletformat|bulletplain> --to <sfbinpack|viriformat|bulletformat> --input <INPUT> --output <OUTPUT>
```

Example with inferred formats:

```bash
cargo run -- convert \
  --input test/ep1.binpack \
  --output out.viri
```

Example with a limit:

```bash
cargo run -- convert \
  --input test/ep1.binpack \
  --output out.viri \
  --limit 1000
```

Example with explicit formats:

```bash
cargo run -- convert \
  --from sfbinpack \
  --to viriformat \
  --input test/ep1.binpack \
  --output out.viri
```

Reverse conversion:

```bash
cargo run -- convert \
  --input out.viri \
  --output roundtrip.binpack
```

Bullet plain-text to bulletformat:

```bash
cargo run -- convert \
  --from bulletplain \
  --to bulletformat \
  --input positions.txt \
  --output positions.bf
```

Where each input line is:

```text
<FEN> | <score> | <result>
```

- `score` is white-relative centipawns
- `result` is white-relative and must be `1.0`, `0.5`, or `0.0`

## Test

```bash
cargo test
```

The test suite includes round-trip checks for both formats and a fixture-based conversion test using `test/ep1.binpack`.

## Read Speed Test

The repo also includes a small benchmark binary for measuring read throughput:

```bash
cargo run --release --bin read-speed -- <INPUT>
```

It supports `viriformat`, `sfbinpack`, and `bulletformat` files.
The input format is inferred from the file extension using the same mapping as `convert`.

Examples:

```bash
cargo run --release --bin read-speed -- test/ep1.binpack
cargo run --release --bin read-speed -- out.viri
cargo run --release --bin read-speed -- positions.bf
```

You can also pass the format explicitly before the path:

```bash
cargo run --release --bin read-speed -- sfbinpack test/ep1.binpack
cargo run --release --bin read-speed -- viriformat out.viri
cargo run --release --bin read-speed -- bulletformat positions.bf
```

For `viriformat` and `sfbinpack`, it reports throughput in games/sec and positions/sec.
For `bulletformat`, it reports positions/sec.

## License

This project is licensed under the GNU General Public License v3.0. See `LICENSE`.
