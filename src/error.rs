use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported conversion from {from} to {to}")]
    UnsupportedConversion {
        from: &'static str,
        to: &'static str,
    },
    #[error("invalid format name: {0}")]
    InvalidFormat(String),
    #[error("I/O error while accessing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("sfbinpack reader error: {0}")]
    SfbinpackReader(#[from] sfbinpack::CompressedReaderError),
    #[error("sfbinpack writer error: {0}")]
    SfbinpackWriter(#[from] sfbinpack::CompressedWriterError),
    #[error("viriformat error: {0}")]
    Viriformat(#[from] anyhow::Error),
    #[error("viriformat data is malformed: {0}")]
    InvalidViriformat(String),
    #[error("bulletformat data is malformed: {0}")]
    InvalidBulletformat(String),
    #[error("invalid FEN: {0}")]
    InvalidFen(String),
    #[error("sfbinpack position could not be expressed as FEN: {0:?}")]
    SfbinpackFen(sfbinpack::chess::position::PositionError),
    #[error("sfbinpack FEN could not be parsed: {0:?}")]
    SfbinpackPosition(sfbinpack::chess::position::PositionError),
    #[error("viriformat board could not be created from FEN: {0}")]
    ViriformatBoard(String),
    #[error("viriformat game outcome {0} cannot be represented in sfbinpack")]
    UnsupportedOutcome(&'static str),
    #[error("viriformat game uses unsupported Chess960 castling rights: {0}")]
    UnsupportedCastling(String),
}

pub type Result<T> = std::result::Result<T, Error>;
