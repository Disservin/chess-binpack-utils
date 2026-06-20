use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::backend;
use crate::error::{Error, Result};
use crate::interrupt;

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

    fn from_path(path: &std::path::Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| Error::InvalidFormat(format!("could not infer format from path: {path:?}")))?;

        match extension {
            "vf" | "viri" | "viriformat" => Ok(Self::Viriformat),
            "sf" | "sfbinpack" | "binpack" => Ok(Self::Sfbinpack),
            "bf" | "bullet" | "bulletformat" => Ok(Self::Bulletformat),
            "txt" | "bulletplain" => Ok(Self::Bulletplain),
            _ => Err(Error::InvalidFormat(format!(
                "unknown file extension for format inference: {extension}"
            ))),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ConvertCommand {
    #[arg(long, value_enum)]
    pub from: Option<Format>,
    #[arg(long, value_enum)]
    pub to: Option<Format>,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub limit: Option<u128>,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Convert(command) => convert(command),
    }
}

fn convert(command: ConvertCommand) -> Result<()> {
    interrupt::install_handler()?;

    let from = command
        .from
        .unwrap_or(Format::from_path(command.input.as_path())?);
    let to = command
        .to
        .unwrap_or(Format::from_path(command.output.as_path())?);

    match (from, to) {
        (Format::Bulletplain, Format::Bulletformat) => {
            backend::bulletformat::convert_text_file(&command.input, &command.output, command.limit)
        }
        (Format::Sfbinpack, Format::Viriformat) => {
            let mut reader = backend::sfbinpack::GameReader::open(&command.input)?;
            let mut writer = backend::viriformat::GameWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer, command.limit)
        }
        (Format::Sfbinpack, Format::Bulletformat) => {
            let mut reader = backend::sfbinpack::GameReader::open(&command.input)?;
            let mut writer = backend::bulletformat::PositionWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer, command.limit)
        }
        (Format::Viriformat, Format::Sfbinpack) => {
            let mut reader = backend::viriformat::GameReader::open(&command.input)?;
            let mut writer = backend::sfbinpack::GameWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer, command.limit)
        }
        (Format::Viriformat, Format::Bulletformat) => {
            let mut reader = backend::viriformat::GameReader::open(&command.input)?;
            let mut writer = backend::bulletformat::PositionWriter::create(&command.output)?;
            backend::stream_convert(&mut reader, &mut writer, command.limit)
        }
        (from, to) => Err(Error::UnsupportedConversion {
            from: from.name(),
            to: to.name(),
        }),
    }
}
