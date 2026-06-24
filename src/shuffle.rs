use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, BufRead, Read, Seek, SeekFrom, Write, ErrorKind};
use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;
use rand::SeedableRng;
use viriformat::dataformat::Game;

use crate::cli::{Backend, ShuffleCommand};
use crate::error::{Error, Result};

const SFBINPACK_MAGIC: [u8; 4] = *b"BINP";
const COPY_BUFFER_SIZE: usize = 1024 * 1024;

pub fn run(command: &ShuffleCommand) -> Result<()> {
    let backend = command
        .backend
        .unwrap_or(Backend::from_path(command.input.as_path())?);
    let outputs = output_paths(&command.output, command.split)?;

    for path in &outputs {
        if path.exists() {
            return Err(Error::OutputExists(path.clone()));
        }
    }

    let mut index = match backend {
        Backend::Sfbinpack => index_sfbinpack(&command.input)?,
        Backend::Viriformat => index_viriformat(&command.input)?,
    };

    if let Some(seed) = command.seed {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        index.shuffle(&mut rng);
    } else {
        let mut rng = rand::thread_rng();
        index.shuffle(&mut rng);
    }

    copy_indexed(&command.input, &outputs, &index)
}

fn output_paths(base: &Path, split: usize) -> Result<Vec<PathBuf>> {
    if split == 0 {
        return Err(Error::InvalidSplitCount(split));
    }

    if split == 1 {
        return Ok(vec![base.to_path_buf()]);
    }

    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let stem = base
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::InvalidFormat(format!("could not infer output basename from path: {base:?}")))?;
    let extension = base.extension().and_then(|value| value.to_str());

    let mut paths = Vec::with_capacity(split);
    for index in 0..split {
        let file_name = match extension {
            Some(extension) => format!("{stem}_{index}.{extension}"),
            None => format!("{stem}_{index}"),
        };
        paths.push(parent.join(file_name));
    }

    Ok(paths)
}

fn index_sfbinpack(path: &Path) -> Result<Vec<(u64, u64)>> {
    let file = File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut index = Vec::new();
    let mut offset = 0u64;

    loop {
        let peek = reader.fill_buf().map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if peek.is_empty() {
            break;
        }

        let mut header = [0u8; 8];
        reader.read_exact(&mut header).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;

        if header[..4] != SFBINPACK_MAGIC {
            return Err(Error::InvalidGameData(format!(
                "invalid sfbinpack chunk magic at byte offset {offset}"
            )));
        }

        let payload_size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as u64;
        reader
            .seek(SeekFrom::Current(payload_size as i64))
            .map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            })?;

        let size = payload_size + header.len() as u64;
        index.push((offset, size));
        offset += size;
    }

    Ok(index)
}

fn index_viriformat(path: &Path) -> Result<Vec<(u64, u64)>> {
    let file = File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut index = Vec::new();

    loop {
        let peek = reader.fill_buf().map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if peek.is_empty() {
            break;
        }

        let start = reader.stream_position().map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;

        match Game::deserialise_from(&mut reader, Vec::new()) {
            Ok(_) => {
                let end = reader.stream_position().map_err(|source| Error::Io {
                    path: path.to_path_buf(),
                    source,
                })?;
                index.push((start, end - start));
            }
            Err(source) if source.kind() == ErrorKind::UnexpectedEof => {
                return Err(Error::Io {
                    path: path.to_path_buf(),
                    source,
                });
            }
            Err(source) => {
                return Err(Error::Io {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
    }

    Ok(index)
}

fn copy_indexed(input: &Path, outputs: &[PathBuf], index: &[(u64, u64)]) -> Result<()> {
    let mut input_file = File::open(input).map_err(|source| Error::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let mut output_files = outputs
        .iter()
        .map(|path| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map(BufWriter::new)
                .map_err(|source| Error::Io {
                    path: path.clone(),
                    source,
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut next_output = 0usize;
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];

    for &(offset, size) in index {
        input_file
            .seek(SeekFrom::Start(offset))
            .map_err(|source| Error::Io {
                path: input.to_path_buf(),
                source,
            })?;
        copy_exact_range(
            &mut input_file,
            &mut output_files[next_output],
            size,
            &mut buffer,
            input,
            &outputs[next_output],
        )?;
        next_output = (next_output + 1) % output_files.len();
    }

    for (path, file) in outputs.iter().zip(output_files.iter_mut()) {
        file.flush().map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
    }

    Ok(())
}

fn copy_exact_range(
    input: &mut File,
    output: &mut BufWriter<File>,
    mut remaining: u64,
    buffer: &mut [u8],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    while remaining > 0 {
        let count = remaining.min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..count]).map_err(|source| Error::Io {
            path: input_path.to_path_buf(),
            source,
        })?;
        output.write_all(&buffer[..count]).map_err(|source| Error::Io {
            path: output_path.to_path_buf(),
            source,
        })?;
        remaining -= count as u64;
    }

    Ok(())
}
