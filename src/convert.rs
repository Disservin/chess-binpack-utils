use sfbinpack::chess::{
    castling_rights::CastlingRights as SfCastlingRights,
    color::Color as SfColor,
    coords::Square as SfSquare,
    r#move::{Move as SfMove, MoveType as SfMoveType},
    piece::Piece as SfPiece,
    piecetype::PieceType as SfPieceType,
};
use viriformat::chess::{
    board::{DrawType, GameOutcome, WinType},
    chessmove::{Move as ViriMove, MoveFlags},
    piece::PieceType as ViriPieceType,
    types::Square as ViriSquare,
};

use crate::error::{Error, Result};
use crate::model::GameResult;

pub fn sf_result_to_game_result(result: i16, side_to_move: SfColor) -> Result<GameResult> {
    match (result, side_to_move) {
        (0, _) => Ok(GameResult::Draw),
        (1, SfColor::White) | (-1, SfColor::Black) => Ok(GameResult::WhiteWin),
        (-1, SfColor::White) | (1, SfColor::Black) => Ok(GameResult::BlackWin),
        _ => Err(Error::InvalidViriformat(format!(
            "unsupported sfbinpack result value: {result}"
        ))),
    }
}

pub fn game_result_to_sf_result(result: GameResult, side_to_move: SfColor) -> i16 {
    match (result, side_to_move) {
        (GameResult::Draw, _) => 0,
        (GameResult::WhiteWin, SfColor::White) | (GameResult::BlackWin, SfColor::Black) => 1,
        (GameResult::WhiteWin, SfColor::Black) | (GameResult::BlackWin, SfColor::White) => -1,
    }
}

pub fn game_result_to_viri_outcome(result: GameResult) -> GameOutcome {
    match result {
        GameResult::WhiteWin => GameOutcome::WhiteWin(WinType::Adjudication),
        GameResult::Draw => GameOutcome::Draw(DrawType::Adjudication),
        GameResult::BlackWin => GameOutcome::BlackWin(WinType::Adjudication),
    }
}

pub fn game_result_to_white_relative(result: GameResult) -> f32 {
    match result {
        GameResult::WhiteWin => 1.0,
        GameResult::Draw => 0.5,
        GameResult::BlackWin => 0.0,
    }
}

pub fn score_to_white_relative(score: i16, fen: &str) -> Result<i16> {
    let side_to_move = fen
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| Error::InvalidFen(format!("missing side-to-move in FEN: {fen}")))?;

    match side_to_move {
        "w" => Ok(score),
        "b" => Ok(-score),
        _ => Err(Error::InvalidFen(format!(
            "invalid side-to-move in FEN: {fen}"
        ))),
    }
}

pub fn sf_move_to_uci(mv: SfMove) -> String {
    mv.as_uci()
}

pub fn uci_to_sf_move(
    uci: &str,
    position: &sfbinpack::chess::position::Position,
) -> Result<SfMove> {
    let bytes = uci.as_bytes();
    if bytes.len() < 4 {
        return Err(Error::InvalidViriformat(format!("invalid UCI move: {uci}")));
    }

    let from = parse_sf_square(&uci[0..2])?;
    let mut to = parse_sf_square(&uci[2..4])?;
    let piece = position.piece_at(from);
    let target = position.piece_at(to);
    let move_type = if bytes.len() == 5 {
        SfMoveType::Promotion
    } else if is_sf_castle(piece, from, to, position.castling_rights()) {
        SfMoveType::Castle
    } else if piece.piece_type() == SfPieceType::Pawn
        && to == position.ep_square()
        && target == SfPiece::none()
    {
        SfMoveType::EnPassant
    } else {
        SfMoveType::Normal
    };

    if move_type == SfMoveType::Castle {
        to = match to.to_string().as_str() {
            "g1" => SfSquare::H1,
            "c1" => SfSquare::A1,
            "g8" => SfSquare::H8,
            "c8" => SfSquare::A8,
            _ => to,
        };
    }

    let promoted_piece = if bytes.len() == 5 {
        parse_sf_promotion(bytes[4] as char, piece.color())?
    } else {
        SfPiece::none()
    };

    Ok(SfMove::new(from, to, move_type, promoted_piece))
}

pub fn viri_move_to_sf_move(mv: ViriMove) -> Result<SfMove> {
    let from = parse_sf_square(&mv.from().to_string())?;
    let to = parse_sf_square(&mv.to().to_string())?;
    if let Some(promo) = mv.promotion_type() {
        return Ok(SfMove::new(
            from,
            to,
            SfMoveType::Promotion,
            SfPiece::new(parse_sf_piece_type(promo)?, SfColor::White),
        ));
    }

    let move_type = if mv.is_castle() {
        SfMoveType::Castle
    } else if mv.is_ep() {
        SfMoveType::EnPassant
    } else {
        SfMoveType::Normal
    };

    Ok(SfMove::new(from, to, move_type, SfPiece::none()))
}

pub fn uci_to_viri_move(uci: &str, board: &viriformat::chess::board::Board) -> Result<ViriMove> {
    let bytes = uci.as_bytes();
    if bytes.len() < 4 {
        return Err(Error::InvalidViriformat(format!("invalid UCI move: {uci}")));
    }

    let from = parse_viri_square(&uci[0..2])?;
    let to = parse_viri_square(&uci[2..4])?;
    let moved_piece = board.piece_at(from).ok_or_else(|| {
        Error::InvalidViriformat(format!("no piece on source square for move {uci}"))
    })?;
    let captured_piece = board.piece_at(to);

    if bytes.len() == 5 {
        let promo = parse_viri_promotion(bytes[4] as char)?;
        return Ok(ViriMove::new_with_promo(from, to, promo));
    }

    if moved_piece.piece_type() == viriformat::chess::piece::PieceType::King {
        let rights = board.castling_rights();
        let is_castle = match (from, to) {
            (ViriSquare::E1, ViriSquare::G1) => rights.wk == Some(ViriSquare::H1),
            (ViriSquare::E1, ViriSquare::C1) => rights.wq == Some(ViriSquare::A1),
            (ViriSquare::E8, ViriSquare::G8) => rights.bk == Some(ViriSquare::H8),
            (ViriSquare::E8, ViriSquare::C8) => rights.bq == Some(ViriSquare::A8),
            _ => false,
        };
        if is_castle {
            let to = match to {
                ViriSquare::G1 => ViriSquare::H1,
                ViriSquare::C1 => ViriSquare::A1,
                ViriSquare::G8 => ViriSquare::H8,
                ViriSquare::C8 => ViriSquare::A8,
                _ => to,
            };
            return Ok(ViriMove::new_with_flags(from, to, MoveFlags::Castle));
        }
    }

    if moved_piece.piece_type() == viriformat::chess::piece::PieceType::Pawn
        && board.ep_sq() == Some(to)
        && captured_piece.is_none()
    {
        return Ok(ViriMove::new_with_flags(from, to, MoveFlags::EnPassant));
    }

    Ok(ViriMove::new(from, to))
}

fn parse_sf_square(square: &str) -> Result<SfSquare> {
    let square = sfbinpack::chess::coords::Square::from_string(square)
        .ok_or_else(|| Error::InvalidViriformat(format!("invalid square: {square}")))?;
    Ok(square)
}

fn parse_viri_square(square: &str) -> Result<ViriSquare> {
    square
        .parse::<ViriSquare>()
        .map_err(|_| Error::InvalidViriformat(format!("invalid square: {square}")))
}

fn parse_sf_promotion(ch: char, color: SfColor) -> Result<SfPiece> {
    let piece_type = match ch {
        'n' => SfPieceType::Knight,
        'b' => SfPieceType::Bishop,
        'r' => SfPieceType::Rook,
        'q' => SfPieceType::Queen,
        _ => {
            return Err(Error::InvalidViriformat(format!(
                "invalid promotion piece: {ch}"
            )));
        }
    };
    Ok(SfPiece::new(piece_type, color))
}

fn is_sf_castle(piece: SfPiece, from: SfSquare, to: SfSquare, rights: SfCastlingRights) -> bool {
    if piece.piece_type() != SfPieceType::King {
        return false;
    }

    match (piece.color(), from, to) {
        (SfColor::White, SfSquare::E1, SfSquare::G1) => rights.contains(SfCastlingRights::WHITE_KING_SIDE),
        (SfColor::White, SfSquare::E1, SfSquare::C1) => rights.contains(SfCastlingRights::WHITE_QUEEN_SIDE),
        (SfColor::Black, SfSquare::E8, SfSquare::G8) => rights.contains(SfCastlingRights::BLACK_KING_SIDE),
        (SfColor::Black, SfSquare::E8, SfSquare::C8) => rights.contains(SfCastlingRights::BLACK_QUEEN_SIDE),
        _ => false,
    }
}

fn parse_viri_promotion(ch: char) -> Result<ViriPieceType> {
    match ch {
        'n' => Ok(ViriPieceType::Knight),
        'b' => Ok(ViriPieceType::Bishop),
        'r' => Ok(ViriPieceType::Rook),
        'q' => Ok(ViriPieceType::Queen),
        _ => Err(Error::InvalidViriformat(format!(
            "invalid promotion piece: {ch}"
        ))),
    }
}

fn parse_sf_piece_type(piece_type: ViriPieceType) -> Result<SfPieceType> {
    match piece_type {
        ViriPieceType::Pawn => Ok(SfPieceType::Pawn),
        ViriPieceType::Knight => Ok(SfPieceType::Knight),
        ViriPieceType::Bishop => Ok(SfPieceType::Bishop),
        ViriPieceType::Rook => Ok(SfPieceType::Rook),
        ViriPieceType::Queen => Ok(SfPieceType::Queen),
        ViriPieceType::King => Ok(SfPieceType::King),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use viriformat::chess::{board::Board, types::Square as ViriSquare};

    #[test]
    fn result_mapping_roundtrips() {
        for side in [SfColor::White, SfColor::Black] {
            for result in [GameResult::WhiteWin, GameResult::Draw, GameResult::BlackWin] {
                let sf = game_result_to_sf_result(result, side);
                assert_eq!(sf_result_to_game_result(sf, side).unwrap(), result);
            }
        }
    }

    #[test]
    fn score_maps_to_white_relative() {
        assert_eq!(
            score_to_white_relative(42, "8/8/8/8/8/8/8/8 w - - 0 1").unwrap(),
            42
        );
        assert_eq!(
            score_to_white_relative(42, "8/8/8/8/8/8/8/8 b - - 0 1").unwrap(),
            -42
        );
    }

    #[test]
    fn uci_to_viri_castle_keeps_king_destination() {
        let mut board = Board::new();
        board
            .set_from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", false)
            .unwrap();

        let mv = uci_to_viri_move("e8g8", &board).unwrap();

        assert!(mv.is_castle());
        assert_eq!(mv.to(), ViriSquare::H8);
    }

    #[test]
    fn viri_castle_maps_to_sf_rook_square() {
        let mv = ViriMove::new_with_flags(ViriSquare::E1, ViriSquare::A1, MoveFlags::Castle);
        let sf_move = viri_move_to_sf_move(mv).unwrap();

        assert_eq!(sf_move.mtype(), SfMoveType::Castle);
        assert_eq!(sf_move.to(), SfSquare::A1);
        assert_eq!(sf_move.as_uci(), "e1c1");
    }

    #[test]
    fn uci_to_viri_non_castle_king_move_stays_normal_without_rights() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/8/4K3 b - - 0 1", false)
            .unwrap();

        let mv = uci_to_viri_move("e8g8", &board).unwrap();

        assert!(!mv.is_castle());
        assert_eq!(mv.to(), ViriSquare::G8);
    }

    #[test]
    fn uci_to_sf_castle_maps_to_sf_rook_square() {
        let position = sfbinpack::chess::position::Position::from_fen(
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        let mv = uci_to_sf_move("e1g1", &position).unwrap();

        assert_eq!(mv.mtype(), SfMoveType::Castle);
        assert_eq!(mv.to(), SfSquare::H1);
        assert_eq!(mv.as_uci(), "e1g1");
    }
}
