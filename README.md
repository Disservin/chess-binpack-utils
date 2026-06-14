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
cargo run -- convert --from <sfbinpack|viriformat|bulletformat|bulletplain> --to <sfbinpack|viriformat|bulletformat> --input <INPUT> --output <OUTPUT>
```

Example:

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
  --from viriformat \
  --to sfbinpack \
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

## License

This project is licensed under the GNU General Public License v3.0. See `LICENSE`.
