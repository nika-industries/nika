//! CLI for the Nika project.

mod nar;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// CLI for the Nika project.
#[derive(Parser, Debug)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Manipulate NAR archives.
  #[command(subcommand)]
  Nar(NarCommand),
}

#[derive(Subcommand, Debug)]
enum NarCommand {
  /// Create a new NAR archive.
  Create(NarCreateArgs),
  /// Extract an existing NAR archive.
  Extract(NarExtractArgs),
}

#[derive(Args, Debug)]
struct NarCreateArgs {
  /// The file system object to archive.
  target: PathBuf,
  /// Sets the path of the output file.
  #[arg(short, long)]
  output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct NarExtractArgs {
  /// The NAR archive to extract.
  target: PathBuf,
  /// Sets the path of the output directory.
  #[arg(short, long)]
  output: Option<PathBuf>,
}

fn main() {
  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::new("info"));
  tracing_subscriber::fmt()
    .without_time()
    .with_env_filter(filter)
    .init();

  let args = Cli::parse();

  match args.command {
    Command::Nar(NarCommand::Create(args)) => {
      let val = crate::nar::create_nar_archive(args);
      if val.is_err() {
        std::process::exit(1);
      }
    }
    Command::Nar(NarCommand::Extract(args)) => {
      let val = crate::nar::extract_nar_archive(args);
      if val.is_err() {
        std::process::exit(1);
      }
    }
  }
}
