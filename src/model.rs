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
