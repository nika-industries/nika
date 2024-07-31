//! CLI for the Nika project.

use clap::{Parser, Subcommand};

/// CLI for the Nika project.
#[derive(Parser, Debug)]
struct Args {
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
  Create,
  /// Extract an existing NAR archive.
  Extract,
}

fn main() {
  let args = Args::parse();
  dbg!(args);
}
