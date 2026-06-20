use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::backend::{self, sfbinpack, viriformat};
use crate::cli::{Backend, ValidateCommand};
use crate::error::{Error, Result};
use crate::interrupt;

pub fn run(command: &ValidateCommand) -> Result<()> {
    let backend = command
        .backend
        .unwrap_or(Backend::from_path(command.input.as_path())?);
    let stdout = std::io::stdout();
    let mut output = stdout.lock();

    validate_to_writer(&mut output, &command.input, backend)
}

pub fn validate_to_writer<W: Write>(output: &mut W, input: &Path, backend: Backend) -> Result<()> {
    let validated = match backend {
        Backend::Sfbinpack => validate_games(sfbinpack::GameReader::open(input)?),
        Backend::Viriformat => validate_games(viriformat::GameReader::open(input)?),
    }?;

    eprintln!();
    writeln!(output, "validated {validated} games").map_err(|source| Error::Io {
        path: "stdout".into(),
        source,
    })?;
    Ok(())
}

fn validate_games<R>(mut reader: R) -> Result<u64>
where
    R: backend::GameReader,
{
    let start = Instant::now();
    let mut last_update = start;
    let mut validated = 0u64;

    while let Some(game) = reader.next_game()? {
        if interrupt::is_requested() {
            break;
        }

        game.validate()?;
        validated += 1;

        let now = Instant::now();
        if now.duration_since(last_update) >= Duration::from_millis(100) {
            print_progress(start, validated);
            last_update = now;
        }
    }

    print_progress(start, validated);

    Ok(validated)
}

fn print_progress(start: Instant, validated: u64) {
    let elapsed = start.elapsed().as_secs_f64();
    let rate = if elapsed == 0.0 {
        0.0
    } else {
        validated as f64 / elapsed
    };

    eprint!(
        "\rValidated: {:>12} games | {:>10.0} games/sec | elapsed: {:>8.1}s",
        validated, rate, elapsed
    );
    io::stderr().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_validated_game_count() {
        let path = std::env::temp_dir().join(format!(
            "chess-binpack-utils-validate-test-{}.binpack",
            std::process::id()
        ));
        let game = crate::model::GameRecord {
            initial_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            result: crate::model::GameResult::WhiteWin,
            positions: vec![crate::model::PositionMoveEval {
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                uci: "e2e4".to_string(),
                score: 24,
                ply: 0,
            }],
        };

        let mut writer = crate::backend::sfbinpack::GameWriter::create(&path).unwrap();
        writer.write_game(&game).unwrap();
        writer.finish();

        let mut output = Vec::new();
        validate_to_writer(&mut output, &path, Backend::Sfbinpack).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "validated 1 games\n");

        let _ = std::fs::remove_file(path);
    }
}
