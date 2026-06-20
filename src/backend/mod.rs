use crate::error::Result;
use crate::interrupt;
use crate::model::GameRecord;

pub mod bulletformat;
pub mod sfbinpack;
pub mod viriformat;

pub trait GameReader {
    fn next_game(&mut self) -> Result<Option<GameRecord>>;
}

pub trait GameWriter {
    fn write_game(&mut self, game: &GameRecord) -> Result<()>;

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

pub fn stream_convert<R, W>(reader: &mut R, writer: &mut W, limit: Option<u128>) -> Result<()>
where
    R: GameReader,
    W: GameWriter,
{
    let mut converted = 0u128;

    while let Some(game) = reader.next_game()? {
        if interrupt::is_requested() {
            break;
        }
        if limit.is_some_and(|limit| converted >= limit) {
            break;
        }

        let remaining = limit.map(|limit| limit - converted);
        let game = truncate_game(game, remaining);
        converted += game.positions.len() as u128;
        writer.write_game(&game)?;
    }

    writer.finish()
}

fn truncate_game(mut game: GameRecord, remaining: Option<u128>) -> GameRecord {
    let Some(remaining) = remaining else {
        return game;
    };

    let keep = remaining.min(game.positions.len() as u128) as usize;
    game.positions.truncate(keep);
    game
}
