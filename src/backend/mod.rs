use crate::error::Result;
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

pub fn stream_convert<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
where
    R: GameReader,
    W: GameWriter,
{
    while let Some(game) = reader.next_game()? {
        writer.write_game(&game)?;
    }

    writer.finish()
}
