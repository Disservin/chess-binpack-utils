use std::{
    env,
    hint::black_box,
    io::{self, Write},
    path::Path,
    time::{Duration, Instant},
};

use bulletformat::{ChessBoard, DataLoader};
use chess_binpack_utils::backend::{sfbinpack, viriformat};

fn main() {
    let (format, path) = parse_args();

    let start = Instant::now();
    let counts = match format {
        Format::Viriformat => {
            benchmark_game_reader(viriformat::GameReader::open(path.as_ref()).unwrap())
        }
        Format::Sfbinpack => {
            benchmark_game_reader(sfbinpack::GameReader::open(path.as_ref()).unwrap())
        }
        Format::Bulletformat => benchmark_bulletformat(path.as_ref()),
    };

    print_progress(
        start,
        counts.items,
        counts.positions,
        format.item_label(),
        format.position_label(),
    );

    let elapsed = start.elapsed();
    println!(
        "average: {:.2} ns/{} | {:.2} ns/{}",
        average_ns(elapsed, counts.items),
        format.item_label_singular(),
        average_ns(elapsed, counts.positions),
        format.position_label_singular(),
    );
}

fn benchmark_game_reader<R>(mut reader: R) -> Counts
where
    R: GameRecordReader,
{
    let start = Instant::now();
    let mut last_update = start;
    let mut counts = Counts::default();

    while let Some(game) = reader.next_game().unwrap() {
        counts.items += 1;
        counts.positions += game.positions.len() as u64;
        black_box(game);

        let now = Instant::now();
        if now.duration_since(last_update) >= Duration::from_millis(100) {
            print_progress(start, counts.items, counts.positions, "games", "positions");
            last_update = now;
        }
    }

    counts
}

fn benchmark_bulletformat(path: &Path) -> Counts {
    let start = Instant::now();
    let mut last_update = start;
    let mut counts = Counts::default();

    DataLoader::<ChessBoard>::new(path, 1)
        .unwrap()
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

    counts
}

fn parse_args() -> (Format, String) {
    let mut args = env::args().skip(1);
    let first = args.next().unwrap_or_else(|| "./test/ep1.viri".to_string());

    match args.next() {
        Some(path) => (Format::parse(&first), path),
        None => (Format::from_path(&first), first),
    }
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

#[derive(Clone, Copy)]
enum Format {
    Viriformat,
    Sfbinpack,
    Bulletformat,
}

impl Format {
    fn parse(value: &str) -> Self {
        match value {
            "vf" | "viri" | "viriformat" => Self::Viriformat,
            "sf" | "sfbinpack" | "binpack" => Self::Sfbinpack,
            "bf" | "bullet" | "bulletformat" => Self::Bulletformat,
            _ => panic!("unknown format: {value}"),
        }
    }

    fn from_path(path: &str) -> Self {
        let extension = Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_else(|| panic!("path has no valid extension: {path}"));

        Self::parse(extension)
    }

    fn item_label(self) -> &'static str {
        match self {
            Self::Viriformat | Self::Sfbinpack => "games",
            Self::Bulletformat => "positions",
        }
    }

    fn item_label_singular(self) -> &'static str {
        match self {
            Self::Viriformat | Self::Sfbinpack => "game",
            Self::Bulletformat => "position",
        }
    }

    fn position_label(self) -> &'static str {
        "positions"
    }

    fn position_label_singular(self) -> &'static str {
        "position"
    }
}

#[derive(Default)]
struct Counts {
    items: u64,
    positions: u64,
}

trait GameRecordReader {
    fn next_game(&mut self) -> anyhow::Result<Option<chess_binpack_utils::model::GameRecord>>;
}

impl GameRecordReader for viriformat::GameReader {
    fn next_game(&mut self) -> anyhow::Result<Option<chess_binpack_utils::model::GameRecord>> {
        viriformat::GameReader::next_game(self).map_err(Into::into)
    }
}

impl GameRecordReader for sfbinpack::GameReader {
    fn next_game(&mut self) -> anyhow::Result<Option<chess_binpack_utils::model::GameRecord>> {
        sfbinpack::GameReader::next_game(self).map_err(Into::into)
    }
}
