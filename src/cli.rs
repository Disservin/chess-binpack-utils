use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::backend;
use crate::error::{Error, Result};

#[derive(Debug, Parser)]
#[command(name = "chess-binpack-utils")]
#[command(about = "Convert chess training data between binpack backends")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Convert(ConvertCommand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Format {
    Sfbinpack,
    Viriformat,
}

impl Format {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sfbinpack => "sfbinpack",
            Self::Viriformat => "viriformat",
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ConvertCommand {
    #[arg(long, value_enum)]
    pub from: Format,
    #[arg(long, value_enum)]
    pub to: Format,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Convert(command) => convert(command),
    }
}

fn convert(command: ConvertCommand) -> Result<()> {
    match (command.from, command.to) {
        (Format::Sfbinpack, Format::Viriformat) => {
            let mut reader = backend::sfbinpack::GameReader::open(&command.input)?;
            let mut writer = backend::viriformat::GameWriter::create(&command.output)?;
            while let Some(game) = reader.next_game()? {
                writer.write_game(&game)?;
            }
            Ok(())
        }
        (Format::Viriformat, Format::Sfbinpack) => {
            let mut reader = backend::viriformat::GameReader::open(&command.input)?;
            let mut writer = backend::sfbinpack::GameWriter::create(&command.output)?;
            while let Some(game) = reader.next_game()? {
                writer.write_game(&game)?;
            }
            writer.finish();
            Ok(())
        }
        (from, to) if from == to => Err(Error::UnsupportedConversion {
            from: from.name(),
            to: to.name(),
        }),
        (from, to) => Err(Error::UnsupportedConversion {
            from: from.name(),
            to: to.name(),
        }),
    }
}
