use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use bulletformat::BulletFormat;
use bulletformat::ChessBoard;

use crate::convert::{game_result_to_white_relative, score_to_white_relative};
use crate::error::{Error, Result};
use crate::model::{GameRecord, PositionMoveEval};

const FLUSH_BATCH_SIZE: usize = 16_384;

pub struct PositionWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    buffer: Vec<ChessBoard>,
}

impl PositionWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            path: path.to_path_buf(),
            writer: BufWriter::new(file),
            buffer: Vec::with_capacity(FLUSH_BATCH_SIZE),
        })
    }

    pub fn write_game(&mut self, game: &GameRecord) -> Result<()> {
        for position in &game.positions {
            self.write_position(position, game.result)?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        self.flush()?;
        self.writer.flush().map_err(|source| Error::Io {
            path: self.path.clone(),
            source,
        })
    }

    fn write_position(
        &mut self,
        position: &PositionMoveEval,
        result: crate::model::GameResult,
    ) -> Result<()> {
        let white_score = score_to_white_relative(position.score, &position.fen)?;
        let white_result = game_result_to_white_relative(result);
        let line = format!("{} | {} | {:.1}", position.fen, white_score, white_result);
        let board = line
            .parse::<ChessBoard>()
            .map_err(Error::InvalidBulletformat)?;
        self.buffer.push(board);

        if self.buffer.len() >= FLUSH_BATCH_SIZE {
            self.flush()?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        ChessBoard::write_to_bin(&mut self.writer, &self.buffer).map_err(|source| Error::Io {
            path: self.path.clone(),
            source,
        })?;
        self.buffer.clear();
        Ok(())
    }
}

pub fn convert_text_file(input: &Path, output: &Path) -> Result<()> {
    let file = File::open(input).map_err(|source| Error::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let mut writer = PositionWriter::create(output)?;

    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|source| Error::Io {
            path: input.to_path_buf(),
            source,
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let board = line.parse::<ChessBoard>().map_err(|source| {
            Error::InvalidBulletformat(format!("line {}: {source}", index + 1))
        })?;
        writer.buffer.push(board);

        if writer.buffer.len() >= FLUSH_BATCH_SIZE {
            writer.flush()?;
        }
    }

    writer.finish()
}
