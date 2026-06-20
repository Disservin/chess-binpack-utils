use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use bulletformat::{BulletFormat, ChessBoard, DataLoader};

use crate::backend::{self, sfbinpack, viriformat};
use crate::cli::{Format, InspectCommand};
use crate::error::{Error, Result};
use crate::interrupt;
use crate::model::GameResult;

pub fn run(command: &InspectCommand) -> Result<()> {
    let format = command
        .format
        .unwrap_or(Format::from_path(command.input.as_path())?);
    let limit = command
        .limit
        .map(|limit| usize::try_from(limit).map_err(|_| Error::InvalidLimit(limit)))
        .transpose()?;
    let stdout = std::io::stdout();
    let mut output = stdout.lock();

    inspect_to_writer(&mut output, &command.input, format, limit)
}

pub fn inspect_to_writer<W: Write>(
    output: &mut W,
    input: &Path,
    format: Format,
    limit: Option<usize>,
) -> Result<()> {
    match format {
        Format::Sfbinpack => inspect_games(output, sfbinpack::GameReader::open(input)?, limit),
        Format::Viriformat => inspect_games(output, viriformat::GameReader::open(input)?, limit),
        Format::Bulletformat => inspect_bulletformat(output, input, limit),
        Format::Bulletplain => inspect_bulletplain(output, input, limit),
    }
}

fn inspect_games<W, R>(output: &mut W, mut reader: R, limit: Option<usize>) -> Result<()>
where
    W: Write,
    R: backend::GameReader,
{
    let mut printed = 0usize;

    while let Some(game) = reader.next_game()? {
        for position in &game.positions {
            if interrupt::is_requested() {
                return Ok(());
            }
            if limit.is_some_and(|limit| printed >= limit) {
                return Ok(());
            }

            writeln!(output, "{}", format_game_entry(position, game.result))
                .map_err(io_error("stdout"))?;
            printed += 1;
        }
    }

    Ok(())
}

fn inspect_bulletformat<W: Write>(
    output: &mut W,
    input: &Path,
    limit: Option<usize>,
) -> Result<()> {
    let mut printed = 0usize;
    let mut write_error = None;

    DataLoader::<ChessBoard>::new(input, 1)
        .map_err(|error| Error::InvalidBulletformat(error.to_string()))?
        .map_positions(|position| {
            if write_error.is_some()
                || interrupt::is_requested()
                || limit.is_some_and(|limit| printed >= limit)
            {
                return;
            }

            if let Err(error) = writeln!(output, "{}", format_bulletformat_entry(position))
                .map_err(io_error("stdout"))
            {
                write_error = Some(error);
                return;
            }
            printed += 1;
        });

    if let Some(error) = write_error {
        return Err(error);
    }

    Ok(())
}

fn inspect_bulletplain<W: Write>(output: &mut W, input: &Path, limit: Option<usize>) -> Result<()> {
    let file = File::open(input).map_err(|source| Error::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let mut printed = 0usize;

    for line in BufReader::new(file).lines() {
        if interrupt::is_requested() {
            return Ok(());
        }
        if limit.is_some_and(|limit| printed >= limit) {
            return Ok(());
        }

        let line = line.map_err(|source| Error::Io {
            path: input.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }

        writeln!(output, "{line}").map_err(io_error("stdout"))?;
        printed += 1;
    }

    Ok(())
}

fn format_game_entry(position: &crate::model::PositionMoveEval, result: GameResult) -> String {
    format!(
        "{} | {} | {} | {} | {}",
        position.fen,
        position.uci,
        position.score,
        position.ply,
        format_result(result)
    )
}

fn format_bulletformat_entry(position: &ChessBoard) -> String {
    format!(
        "{} w - - 0 1 | {} | {:.1}",
        bulletformat_board_fen(position),
        position.score(),
        position.result()
    )
}

fn bulletformat_board_fen(position: &ChessBoard) -> String {
    let mut board = ['1'; 64];

    for (piece, square) in (*position).into_iter() {
        board[usize::from(square)] = bulletformat_piece_char(piece);
    }

    let mut fen = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0usize;
        for file in 0..8 {
            let piece = board[rank * 8 + file];
            if piece == '1' {
                empty += 1;
                continue;
            }

            if empty > 0 {
                fen.push(char::from_digit(empty as u32, 10).expect("empty run is 1..=8"));
                empty = 0;
            }
            fen.push(piece);
        }

        if empty > 0 {
            fen.push(char::from_digit(empty as u32, 10).expect("empty run is 1..=8"));
        }
        if rank > 0 {
            fen.push('/');
        }
    }

    fen
}

fn bulletformat_piece_char(piece: u8) -> char {
    match piece {
        0 => 'P',
        1 => 'N',
        2 => 'B',
        3 => 'R',
        4 => 'Q',
        5 => 'K',
        8 => 'p',
        9 => 'n',
        10 => 'b',
        11 => 'r',
        12 => 'q',
        13 => 'k',
        _ => '?',
    }
}

fn format_result(result: GameResult) -> &'static str {
    match result {
        GameResult::WhiteWin => "1-0",
        GameResult::Draw => "1/2-1/2",
        GameResult::BlackWin => "0-1",
    }
}

fn io_error(path: &'static str) -> impl FnOnce(std::io::Error) -> Error {
    move |source| Error::Io {
        path: path.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_game_entry_as_single_line() {
        let entry = crate::model::PositionMoveEval {
            fen: "startpos-fen".to_string(),
            uci: "e2e4".to_string(),
            score: 24,
            ply: 0,
        };

        assert_eq!(
            format_game_entry(&entry, GameResult::WhiteWin),
            "startpos-fen | e2e4 | 24 | 0 | 1-0"
        );
    }

    #[test]
    fn formats_bulletformat_entry_as_text_line() {
        let entry: ChessBoard = "8/8/8/8/8/8/8/K6k w - - 0 1 | 24 | 1.0".parse().unwrap();

        assert_eq!(
            format_bulletformat_entry(&entry),
            "8/8/8/8/8/8/8/K6k w - - 0 1 | 24 | 1.0"
        );
    }
}
