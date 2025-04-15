
use panutils::cmdline::*;
use panutils::fastixe;
use clap::Parser;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Fastixe(fastixe_args) => fastixe::fastixe(fastixe_args),
    }

}
