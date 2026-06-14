pub mod backend;
pub mod cli;
pub mod convert;
pub mod error;
pub mod model;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    use bulletformat::{ChessBoard, DataLoader};

    use crate::backend::{sfbinpack, viriformat};
    use crate::model::{GameRecord, GameResult, PositionMoveEval};

    #[test]
    fn viriformat_read_write_roundtrip() {
        let path = temp_path("viri");
        let games = sample_games();

        write_viriformat_games(&path, &games);
        let parsed = read_viriformat_games(&path);

        assert_eq!(parsed, games);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sfbinpack_read_write_roundtrip() {
        let path = temp_path("binpack");
        let games = sample_games();

        write_sfbinpack_games(&path, &games);
        let parsed = read_sfbinpack_games(&path);

        assert_eq!(parsed, games);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn streaming_convert_sfbinpack_to_viriformat() {
        let input = temp_path("binpack");
        let output = temp_path("viri");
        let games = sample_games();

        write_sfbinpack_games(&input, &games);

        let mut reader = sfbinpack::GameReader::open(&input).unwrap();
        let mut writer = viriformat::GameWriter::create(&output).unwrap();
        while let Some(game) = reader.next_game().unwrap() {
            writer.write_game(&game).unwrap();
        }

        let parsed = read_viriformat_games(&output);
        assert_eq!(parsed, games);

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn streaming_convert_sfbinpack_to_bulletformat() {
        let input = temp_path("binpack");
        let output = temp_path("bf");
        let games = sample_games();

        write_sfbinpack_games(&input, &games);

        let mut reader = sfbinpack::GameReader::open(&input).unwrap();
        let mut writer = crate::backend::bulletformat::PositionWriter::create(&output).unwrap();
        while let Some(game) = reader.next_game().unwrap() {
            writer.write_game(&game).unwrap();
        }
        writer.finish().unwrap();

        let parsed = read_bulletformat_positions(&output);
        let expected = vec![
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 | 24 | 1.0",
            )
            .unwrap(),
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1 | 18 | 1.0",
            )
            .unwrap(),
            ChessBoard::from_str(
                "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2 | 31 | 1.0",
            )
            .unwrap(),
        ];

        assert_eq!(parsed, expected);

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn streaming_convert_viriformat_to_bulletformat() {
        let input = temp_path("viri");
        let output = temp_path("bf");
        let games = sample_games();

        write_viriformat_games(&input, &games);

        let mut reader = viriformat::GameReader::open(&input).unwrap();
        let mut writer = crate::backend::bulletformat::PositionWriter::create(&output).unwrap();
        while let Some(game) = reader.next_game().unwrap() {
            writer.write_game(&game).unwrap();
        }
        writer.finish().unwrap();

        let parsed = read_bulletformat_positions(&output);
        let expected = vec![
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 | 24 | 1.0",
            )
            .unwrap(),
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1 | 18 | 1.0",
            )
            .unwrap(),
            ChessBoard::from_str(
                "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2 | 31 | 1.0",
            )
            .unwrap(),
        ];

        assert_eq!(parsed, expected);

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn convert_text_to_bulletformat() {
        let input = temp_path("txt");
        let output = temp_path("bf");

        std::fs::write(
            &input,
            concat!(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 | 24 | 1.0\n",
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1 | 18 | 1.0\n",
            ),
        )
        .unwrap();

        crate::backend::bulletformat::convert_text_file(&input, &output).unwrap();

        let parsed = read_bulletformat_positions(&output);
        let expected = vec![
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 | 24 | 1.0",
            )
            .unwrap(),
            ChessBoard::from_str(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1 | 18 | 1.0",
            )
            .unwrap(),
        ];

        assert_eq!(parsed, expected);

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn fixture_sfbinpack_roundtrips_through_viriformat() {
        let input = PathBuf::from("test/ep1.binpack");
        let viriformat_output = temp_path("viri");
        let roundtrip_output = temp_path("binpack");

        let mut reader = sfbinpack::GameReader::open(&input).unwrap();
        let mut writer = viriformat::GameWriter::create(&viriformat_output).unwrap();
        while let Some(game) = reader.next_game().unwrap() {
            writer.write_game(&game).unwrap();
        }

        let mut reader = viriformat::GameReader::open(&viriformat_output).unwrap();
        let mut writer = sfbinpack::GameWriter::create(&roundtrip_output).unwrap();
        while let Some(game) = reader.next_game().unwrap() {
            writer.write_game(&game).unwrap();
        }
        writer.finish();

        let original = std::fs::read(&input).unwrap();
        let roundtrip = std::fs::read(&roundtrip_output).unwrap();
        assert_eq!(roundtrip, original);

        let _ = std::fs::remove_file(viriformat_output);
        let _ = std::fs::remove_file(roundtrip_output);
    }

    fn sample_games() -> Vec<GameRecord> {
        vec![GameRecord {
            initial_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            result: GameResult::WhiteWin,
            positions: vec![
                PositionMoveEval {
                    fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                    uci: "e2e4".to_string(),
                    score: 24,
                    ply: 0,
                },
                PositionMoveEval {
                    fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string(),
                    uci: "e7e5".to_string(),
                    score: -18,
                    ply: 1,
                },
                PositionMoveEval {
                    fen: "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".to_string(),
                    uci: "g1f3".to_string(),
                    score: 31,
                    ply: 2,
                },
            ],
        }]
    }

    fn read_sfbinpack_games(path: &std::path::Path) -> Vec<GameRecord> {
        let mut reader = sfbinpack::GameReader::open(path).unwrap();
        let mut games = Vec::new();
        while let Some(game) = reader.next_game().unwrap() {
            games.push(game);
        }
        games
    }

    fn write_sfbinpack_games(path: &std::path::Path, games: &[GameRecord]) {
        let mut writer = sfbinpack::GameWriter::create(path).unwrap();
        for game in games {
            writer.write_game(game).unwrap();
        }
        writer.finish();
    }

    fn read_viriformat_games(path: &std::path::Path) -> Vec<GameRecord> {
        let mut reader = viriformat::GameReader::open(path).unwrap();
        let mut games = Vec::new();
        while let Some(game) = reader.next_game().unwrap() {
            games.push(game);
        }
        games
    }

    fn write_viriformat_games(path: &std::path::Path, games: &[GameRecord]) {
        let mut writer = viriformat::GameWriter::create(path).unwrap();
        for game in games {
            writer.write_game(game).unwrap();
        }
    }

    fn read_bulletformat_positions(path: &std::path::Path) -> Vec<ChessBoard> {
        let mut positions = Vec::new();
        DataLoader::<ChessBoard>::new(path, 1)
            .unwrap()
            .map_positions(|position| positions.push(*position));
        positions
    }

    fn temp_path(extension: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "chess-binpack-utils-{suffix}-{}.{}",
            std::process::id(),
            extension
        ))
    }
}
