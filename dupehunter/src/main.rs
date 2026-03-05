use dupehunter::{Args, Scanner};
use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut scanner = Scanner::new(args);
    scanner.scan()?;
    scanner.report();
    Ok(())
}
