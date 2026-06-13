use clap::Parser;

fn main() {
    let cli = chess_binpack_utils::cli::Cli::parse();
    if let Err(error) = chess_binpack_utils::cli::run(cli) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
