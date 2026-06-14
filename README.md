# chess-binpack-utils

Small Rust CLI for converting chess training data between `sfbinpack` and `viriformat`.

## What It Does

- Reads game-oriented training data from one backend
- Streams it into the other backend
- Preserves move sequence, evaluation score, ply, initial FEN, and game result

Supported conversions:

- `sfbinpack` -> `viriformat`
- `viriformat` -> `sfbinpack`

## Limitations

- Converting a format to itself is rejected
- `viriformat` input using Chess960-style castling rights is not supported when writing `viriformat` output
- `viriformat` game outcomes must be representable as win, draw, or loss in `sfbinpack`

## Build

```bash
cargo build
```

## Usage

```bash
cargo run -- convert --from <sfbinpack|viriformat> --to <sfbinpack|viriformat> --input <INPUT> --output <OUTPUT>
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

## Test

```bash
cargo test
```

The test suite includes round-trip checks for both formats and a fixture-based conversion test using `test/ep1.binpack`.

## License

This project is licensed under the GNU General Public License v3.0. See `LICENSE`.
