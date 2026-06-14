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
    Bulletformat,
    Bulletplain,
}

impl Format {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sfbinpack => "sfbinpack",
            Self::Viriformat => "viriformat",
            Self::Bulletformat => "bulletformat",
            Self::Bulletplain => "bulletplain",
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
        (Format::Bulletplain, Format::Bulletformat) => {
            backend::bulletformat::convert_text_file(&command.input, &command.output)
        }
        (Format::Sfbinpack, Format::Viriformat) => {
            let mut reader = backend::sfbinpack::GameReader::open(&command.input)?;
            let mut writer = backend::viriformat::GameWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer)
        }
        (Format::Sfbinpack, Format::Bulletformat) => {
            let mut reader = backend::sfbinpack::GameReader::open(&command.input)?;
            let mut writer = backend::bulletformat::PositionWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer)
        }
        (Format::Viriformat, Format::Sfbinpack) => {
            let mut reader = backend::viriformat::GameReader::open(&command.input)?;
            let mut writer = backend::sfbinpack::GameWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer)
        }
        (Format::Viriformat, Format::Bulletformat) => {
            let mut reader = backend::viriformat::GameReader::open(&command.input)?;
            let mut writer = backend::bulletformat::PositionWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer)
        }
        (from, to) => Err(Error::UnsupportedConversion {
            from: from.name(),
            to: to.name(),
        }),
    }
}
