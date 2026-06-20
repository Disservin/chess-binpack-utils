use std::hint::black_box;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use bulletformat::{ChessBoard, DataLoader};

use crate::backend::{self, sfbinpack, viriformat};
use crate::cli::Format;
use crate::error::Result;
use crate::model::GameRecord;

pub fn run(command: &crate::cli::BenchmarkCommand) -> Result<()> {
    let format = command
        .format
        .unwrap_or(Format::from_path(command.input.as_path())?);

    let start = Instant::now();
    let counts = match format {
        Format::Viriformat => benchmark_game_reader(viriformat::GameReader::open(&command.input)?),
        Format::Sfbinpack => benchmark_game_reader(sfbinpack::GameReader::open(&command.input)?),
        Format::Bulletformat => benchmark_bulletformat(&command.input),
        Format::Bulletplain => benchmark_bulletplain(&command.input),
    }?;

    print_progress(
        start,
        counts.items,
        counts.positions,
        format.item_label(),
        format.position_label(),
    );
    eprintln!();

    let elapsed = start.elapsed();
    println!(
        "average: {:.2} ns/{} | {:.2} ns/{}",
        average_ns(elapsed, counts.items),
        format.item_label_singular(),
        average_ns(elapsed, counts.positions),
        format.position_label_singular(),
    );

    Ok(())
}

fn benchmark_game_reader<R>(mut reader: R) -> Result<Counts>
where
    R: backend::GameReader,
{
    let start = Instant::now();
    let mut last_update = start;
    let mut counts = Counts::default();

    while let Some(game) = reader.next_game()? {
        counts.items += 1;
        counts.positions += game.positions.len() as u64;
        black_box(game);

        let now = Instant::now();
        if now.duration_since(last_update) >= Duration::from_millis(100) {
            print_progress(start, counts.items, counts.positions, "games", "positions");
            last_update = now;
        }
    }

    Ok(counts)
}

fn benchmark_bulletformat(path: &Path) -> Result<Counts> {
    let start = Instant::now();
    let mut last_update = start;
    let mut counts = Counts::default();

    DataLoader::<ChessBoard>::new(path, 1)
        .map_err(|error| crate::error::Error::InvalidBulletformat(error.to_string()))?
        .map_positions(|position| {
            counts.items += 1;
            counts.positions += 1;
            black_box(position);

            let now = Instant::now();
            if now.duration_since(last_update) >= Duration::from_millis(100) {
                print_progress(
                    start,
                    counts.items,
                    counts.positions,
                    "positions",
                    "positions",
                );
                last_update = now;
            }
        });

    Ok(counts)
}

fn benchmark_bulletplain(path: &Path) -> Result<Counts> {
    let text = std::fs::read_to_string(path).map_err(|source| crate::error::Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let start = Instant::now();
    let mut last_update = start;
    let mut counts = Counts::default();

    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        counts.items += 1;
        counts.positions += 1;
        black_box(line);

        let now = Instant::now();
        if now.duration_since(last_update) >= Duration::from_millis(100) {
            print_progress(
                start,
                counts.items,
                counts.positions,
                "positions",
                "positions",
            );
            last_update = now;
        }
    }

    Ok(counts)
}

fn print_progress(
    start: Instant,
    items: u64,
    positions: u64,
    item_label: &str,
    position_label: &str,
) {
    let elapsed = start.elapsed().as_secs_f64();
    let item_speed = items as f64 / elapsed;
    let position_speed = positions as f64 / elapsed;

    eprint!(
        "\rProcessed: {:>12} {} | {:>12} {} | {:>10.0} {}/sec | {:>12.0} {}/sec | elapsed: {:>8.1}s",
        items,
        item_label,
        positions,
        position_label,
        item_speed,
        item_label,
        position_speed,
        position_label,
        elapsed
    );

    io::stderr().flush().unwrap();
}

fn average_ns(elapsed: Duration, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        elapsed.as_nanos() as f64 / count as f64
    }
}

#[derive(Default)]
struct Counts {
    items: u64,
    positions: u64,
}

#[allow(dead_code)]
fn _black_box_game_record(game: GameRecord) {
    black_box(game);
}
