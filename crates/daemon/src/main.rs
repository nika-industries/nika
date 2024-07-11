#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::fmt::init();

  println!();
  for line in art::ascii_art!("../../media/ascii_logo.png").lines() {
    println!("{}", line);
  }
  println!();

  Ok(())
}
