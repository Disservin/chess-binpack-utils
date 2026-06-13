use std::fs::File;
use std::path::Path;

use sfbinpack::{CompressedTrainingDataEntryReader, CompressedTrainingDataEntryWriter};

use crate::convert::{game_result_to_sf_result, sf_move_to_uci, uci_to_sf_move};
use crate::error::{Error, Result};
use crate::model::{GameRecord, PositionMoveEval};

pub struct GameReader {
    reader: CompressedTrainingDataEntryReader<File>,
    current: Vec<sfbinpack::TrainingDataEntry>,
}

impl GameReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let reader = CompressedTrainingDataEntryReader::new(file)?;
        Ok(Self {
            reader,
            current: Vec::new(),
        })
    }

    pub fn next_game(&mut self) -> Result<Option<GameRecord>> {
        while self.reader.has_next() {
            let continuation = !self.current.is_empty() && self.reader.is_next_entry_continuation();
            let entry = self.reader.next();

            if !continuation && !self.current.is_empty() {
                let game = build_game_record(&self.current)?;
                self.current.clear();
                self.current.push(entry);
                return Ok(Some(game));
            }

            self.current.push(entry);
        }

        if self.current.is_empty() {
            return Ok(None);
        }

        let game = build_game_record(&self.current)?;
        self.current.clear();
        Ok(Some(game))
    }
}

pub struct GameWriter {
    writer: CompressedTrainingDataEntryWriter<File>,
}

impl GameWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let writer = CompressedTrainingDataEntryWriter::new(file)?;
        Ok(Self { writer })
    }

    pub fn write_game(&mut self, game: &GameRecord) -> Result<()> {
        let mut position = sfbinpack::chess::position::Position::from_fen(&game.initial_fen)
            .map_err(Error::SfbinpackPosition)?;

        for item in &game.positions {
            let mv = uci_to_sf_move(&item.uci, &position)?;
            let result = game_result_to_sf_result(game.result, position.side_to_move());
            let entry = sfbinpack::TrainingDataEntry {
                pos: position,
                mv,
                score: item.score,
                ply: item.ply,
                result,
            };
            self.writer.write_entry(&entry)?;
            position.do_move(mv);
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        self.writer.flush_and_end();
    }
}

fn build_game_record(entries: &[sfbinpack::TrainingDataEntry]) -> Result<GameRecord> {
    let first = entries
        .first()
        .ok_or_else(|| Error::InvalidViriformat("empty sfbinpack game".to_string()))?;
    let initial_fen = first.pos.fen().map_err(Error::SfbinpackFen)?;
    let result = crate::convert::sf_result_to_game_result(first.result, first.pos.side_to_move())?;
    let mut positions = Vec::with_capacity(entries.len());

    for entry in entries {
        positions.push(PositionMoveEval {
            fen: entry.pos.fen().map_err(Error::SfbinpackFen)?,
            uci: sf_move_to_uci(entry.mv),
            score: entry.score,
            ply: entry.ply,
        });
    }

    Ok(GameRecord {
        initial_fen,
        result,
        positions,
    })
}
