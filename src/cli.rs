use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::backend;
use crate::benchmark;
use crate::error::{Error, Result};
use crate::inspect;
use crate::interrupt;
use crate::unique;

#[derive(Debug, Parser)]
#[command(name = "chess-binpack-utils")]
#[command(about = "General-purpose tool for working with chess binpack data")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Convert(ConvertCommand),
    Unique(UniqueCommand),
    Inspect(InspectCommand),
    Benchmark(BenchmarkCommand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Backend {
    Sfbinpack,
    Viriformat,
}

impl Backend {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sfbinpack => "sfbinpack",
            Self::Viriformat => "viriformat",
        }
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| {
                Error::InvalidFormat(format!("could not infer backend from path: {path:?}"))
            })?;

        match extension {
            "vf" | "viri" | "viriformat" => Ok(Self::Viriformat),
            "sf" | "sfbinpack" | "binpack" => Ok(Self::Sfbinpack),
            _ => Err(Error::InvalidFormat(format!(
                "unknown file extension for backend inference: {extension}"
            ))),
        }
    }
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

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| {
                Error::InvalidFormat(format!("could not infer format from path: {path:?}"))
            })?;

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

    pub const fn item_label(self) -> &'static str {
        match self {
            Self::Viriformat | Self::Sfbinpack => "games",
            Self::Bulletformat | Self::Bulletplain => "positions",
        }
    }

    pub const fn item_label_singular(self) -> &'static str {
        match self {
            Self::Viriformat | Self::Sfbinpack => "game",
            Self::Bulletformat | Self::Bulletplain => "position",
        }
    }

    pub const fn position_label(self) -> &'static str {
        "positions"
    }

    pub const fn position_label_singular(self) -> &'static str {
        "position"
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

#[derive(Debug, clap::Args)]
pub struct UniqueCommand {
    #[arg(long, value_enum)]
    pub backend: Option<Backend>,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub limit: Option<u128>,
}

#[derive(Debug, clap::Args)]
pub struct BenchmarkCommand {
    #[arg(long, value_enum)]
    pub format: Option<Format>,
    #[arg(long)]
    pub input: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct InspectCommand {
    #[arg(long, value_enum)]
    pub format: Option<Format>,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub limit: Option<u128>,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Convert(command) => convert(command),
        Command::Unique(command) => unique_command(command),
        Command::Inspect(command) => inspect_command(command),
        Command::Benchmark(command) => benchmark_command(command),
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

fn unique_command(command: UniqueCommand) -> Result<()> {
    interrupt::install_handler()?;

    let backend = command
        .backend
        .unwrap_or(Backend::from_path(command.input.as_path())?);
    let limit = command
        .limit
        .map(|limit| usize::try_from(limit).map_err(|_| Error::InvalidLimit(limit)))
        .transpose()?;
    let unique = unique::unique_positions_from_path(&command.input, limit, backend)?;
    println!("{unique}");
    Ok(())
}

fn benchmark_command(command: BenchmarkCommand) -> Result<()> {
    interrupt::install_handler()?;
    benchmark::run(&command)
}

fn inspect_command(command: InspectCommand) -> Result<()> {
    interrupt::install_handler()?;
    inspect::run(&command)
}
