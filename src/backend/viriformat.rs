use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::Path;

use viriformat::chess::board::{Board, DrawType, GameOutcome, WinType};
use viriformat::dataformat::Game;

use crate::backend;
use crate::convert::{
    game_result_to_viri_outcome, sf_move_to_uci, uci_to_viri_move, viri_move_to_sf_move,
};
use crate::error::{Error, Result};
use crate::model::{GameRecord, PositionMoveEval};

pub struct GameReader {
    path: Box<Path>,
    reader: BufReader<File>,
}

impl GameReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            path: path.into(),
            reader: BufReader::new(file),
        })
    }

    pub fn next_game(&mut self) -> Result<Option<GameRecord>> {
        match Game::deserialise_from(&mut self.reader, Vec::new()) {
            Ok(game) => Ok(Some(game_to_record(game)?)),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(source) => Err(Error::Io {
                path: self.path.to_path_buf(),
                source,
            }),
        }
    }
}

impl backend::GameReader for GameReader {
    fn next_game(&mut self) -> Result<Option<GameRecord>> {
        Self::next_game(self)
    }
}

pub struct GameWriter {
    path: Box<Path>,
    file: File,
}

impl GameWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            path: path.into(),
            file,
        })
    }

    pub fn write_game(&mut self, record: &GameRecord) -> Result<()> {
        let mut board = Board::new();
        board
            .set_from_fen(&record.initial_fen, false)
            .map_err(|error| Error::ViriformatBoard(error.to_string()))?;
        validate_standard_castling(&board)?;

        let mut game = Game::new(&board);
        game.set_outcome(game_result_to_viri_outcome(record.result));

        for item in &record.positions {
            let mv = uci_to_viri_move(&item.uci, &board)?;
            game.add_move(mv, item.score);
            if !board.make_move_simple(mv) {
                return Err(Error::InvalidViriformat(
                    "cannot replay move while writing viriformat output".to_string(),
                ));
            }
        }

        game.serialise_into(&mut self.file)
            .map_err(|source| Error::Io {
                path: self.path.to_path_buf(),
                source,
            })?;
        Ok(())
    }
}

impl backend::GameWriter for GameWriter {
    fn write_game(&mut self, game: &GameRecord) -> Result<()> {
        Self::write_game(self, game)
    }
}

fn game_to_record(game: Game) -> Result<GameRecord> {
    let mut board = game.initial_position();
    let initial_fen = board.to_string();
    let result = match game.initial_position.unpack().2 {
        2 => crate::model::GameResult::WhiteWin,
        1 => crate::model::GameResult::Draw,
        0 => crate::model::GameResult::BlackWin,
        _ => {
            return Err(Error::InvalidViriformat(
                "viriformat game has an invalid packed result".to_string(),
            ));
        }
    };
    let mut positions = Vec::with_capacity(game.moves.len());

    for (mv, eval) in game.moves {
        let sf_move = viri_move_to_sf_move(mv)?;
        positions.push(PositionMoveEval {
            fen: board.to_string(),
            uci: sf_move_to_uci(sf_move),
            score: eval.get(),
            ply: board.ply() as u16,
        });
        if !board.make_move_simple(mv) {
            return Err(Error::InvalidViriformat(
                "viriformat game contains an illegal move sequence".to_string(),
            ));
        }
    }

    Ok(GameRecord {
        initial_fen,
        result,
        positions,
    })
}

fn validate_standard_castling(board: &Board) -> Result<()> {
    let rights = board.castling_rights();
    let valid = rights.wk.is_none_or(|sq| sq.to_string() == "h1")
        && rights.wq.is_none_or(|sq| sq.to_string() == "a1")
        && rights.bk.is_none_or(|sq| sq.to_string() == "h8")
        && rights.bq.is_none_or(|sq| sq.to_string() == "a8");

    if valid {
        return Ok(());
    }

    Err(Error::UnsupportedCastling(rights.display(true).to_string()))
}

#[allow(dead_code)]
fn _default_draw() -> GameOutcome {
    GameOutcome::Draw(DrawType::Adjudication)
}

#[allow(dead_code)]
fn _default_win() -> GameOutcome {
    GameOutcome::WhiteWin(WinType::Adjudication)
}
