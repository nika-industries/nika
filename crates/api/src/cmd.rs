use clap::Parser;

#[derive(Parser, Debug)]
pub struct RuntimeConfig {
  #[arg(short = 'a', long = "address", default_value = "0.0.0.0")]
  pub bind_address:      String,
  #[arg(short = 'p', long = "port", default_value = "3000")]
  pub bind_port:         u16,
  #[arg(long, action)]
  pub mock_temp_storage: bool,
  #[arg(long, action)]
  pub chrome_tracing:    bool,
}
