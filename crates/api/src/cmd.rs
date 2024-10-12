use clap::{Args, Parser, Subcommand, ValueEnum};

// #[derive(Parser, Debug)]
// pub struct RuntimeConfig {
//   #[arg(short = 'a', long = "address", default_value = "0.0.0.0")]
//   pub bind_address:      String,
//   #[arg(short = 'p', long = "port", default_value = "3000")]
//   pub bind_port:         u16,
//   #[arg(long, action)]
//   pub mock_temp_storage: bool,
//   #[arg(long, action)]
//   pub chrome_tracing:    bool,
// }

#[derive(Parser, Debug)]
pub struct RuntimeConfig {
  #[command(subcommand)]
  pub command:           Commands,
  #[arg(long, action)]
  pub mock_temp_storage: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
  /// Starts the API server with the given host and port
  Start {
    #[arg(short = 'a', long = "address", default_value = "0.0.0.0")]
    bind_address:   String,
    #[arg(short = 'p', long = "port", default_value = "3000")]
    bind_port:      u16,
    #[arg(long, action)]
    chrome_tracing: bool,
  },
  /// Checks the health of the API server services and dumps the report to
  /// stdout
  Health,
}
