use shakmaty::{CastlingMode, Chess, Position, fen::Fen, uci::UciMove};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    WhiteWin,
    Draw,
    BlackWin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionMoveEval {
    pub fen: String,
    pub uci: String,
    pub score: i16,
    pub ply: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameRecord {
    pub initial_fen: String,
    pub result: GameResult,
    pub positions: Vec<PositionMoveEval>,
}

impl GameRecord {
    pub fn validate(&self) -> Result<()> {
        let fen = Fen::from_ascii(self.initial_fen.as_bytes())
            .map_err(|_| Error::InvalidFen(self.initial_fen.clone()))?;
        let mut position: Chess = fen.into_position(CastlingMode::Standard).map_err(|_| {
            Error::InvalidGameData(format!("invalid position from FEN: {}", self.initial_fen))
        })?;

        for (index, item) in self.positions.iter().enumerate() {
            let uci: UciMove = item.uci.parse().map_err(|error| {
                Error::InvalidGameData(format!(
                    "invalid UCI move at index {index} ({}) : {error}",
                    item.uci
                ))
            })?;
            let chess_move = uci.to_move(&position).map_err(|error| {
                Error::InvalidGameData(format!(
                    "illegal move at index {index} ({}) : {error}",
                    item.uci
                ))
            })?;
            position.play_unchecked(&chess_move);
        }

        Ok(())
    }
}
