use clap::Parser;

fn main() {
    let cli = chess_binpack_utils::cli::Cli::parse();
    if let Err(error) = chess_binpack_utils::cli::run(cli) {
        if matches!(
            &error,
            chess_binpack_utils::error::Error::Io { source, .. }
                if source.kind() == std::io::ErrorKind::BrokenPipe
        ) {
            return;
        }
        eprintln!("{error}");
        std::process::exit(1);
    }
}
